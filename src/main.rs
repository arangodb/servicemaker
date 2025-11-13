use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;

/// A tool to wrap Python projects as Docker services
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the project
    #[arg(long)]
    name: Option<String>,

    /// Path to the folder containing the Python project
    #[arg(long)]
    project_home: Option<PathBuf>,

    /// Base Docker image
    #[arg(long, default_value = "debian:trixie-slim")]
    base_image: String,

    /// Exposed port number
    #[arg(long)]
    port: Option<u16>,

    /// Docker image name to push
    #[arg(long)]
    image_name: Option<String>,

    /// Whether to push the image
    #[arg(long, default_value = "false")]
    push: bool,

    /// URL of the Docker registry (default: Docker Hub)
    #[arg(long)]
    registry: Option<String>,

    /// Name of the Python script to run (relative to project home)
    #[arg(long)]
    entrypoint: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Args::parse();

    // Get project home first (prompt if needed)
    if args.project_home.is_none() {
        let path_str = prompt("Project home path")?;
        args.project_home = Some(PathBuf::from(path_str));
    }
    let project_home = args.project_home.as_ref().unwrap();

    // Validate project home exists
    if !project_home.exists() {
        return Err(format!("Project home does not exist: {}", project_home.display()).into());
    }

    // Try to get name from pyproject.toml if not provided on command line
    if args.name.is_none()
        && let Ok(name) = read_name_from_pyproject(project_home)
    {
        args.name = Some(name);
    }

    // Prompt for name if still not set
    if args.name.is_none() {
        args.name = Some(prompt("Project name")?);
    }

    let project_dir = project_home.file_name().unwrap().to_str().unwrap();

    if args.port.is_none() {
        let port_str = prompt("Exposed port number")?;
        args.port = Some(port_str.parse().expect("Invalid port number"));
    }

    if args.image_name.is_none() {
        args.image_name = Some(prompt("Docker image name")?);
    }

    if args.entrypoint.is_none() {
        args.entrypoint = Some(prompt("Python entrypoint script (e.g., main.py)")?);
    }

    let name = args.name.as_ref().unwrap();
    let project_home = args.project_home.as_ref().unwrap();
    let image_name = args.image_name.as_ref().unwrap();
    let entrypoint = args.entrypoint.as_ref().unwrap();
    let port = args.port.unwrap();

    println!("\n=== Configuration ===");
    println!("Project name: {}", name);
    println!("Project home: {}", project_home.display());
    println!("Project directory name: {}", project_dir);
    println!("Base image: {}", args.base_image);
    println!("Port: {}", port);
    println!("Image name: {}", image_name);
    println!("Entrypoint: {}", entrypoint);
    println!("Push: {}", args.push);
    if let Some(registry) = &args.registry {
        println!("Registry: {}", registry);
    }
    println!("=====================\n");

    // Create temporary directory
    let temp_dir =
        std::env::temp_dir().join(format!("servicemaker-{}-{}", name, std::process::id()));
    println!("Creating temporary directory: {}", temp_dir.display());

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // Copy project directory to temp directory
    let project_dest = temp_dir.join("project");
    println!(
        "Copying project from {} to {}",
        project_home.display(),
        project_dest.display()
    );
    copy_dir_recursive(project_home, &project_dest)?;

    // Read and modify Dockerfile template
    let dockerfile_template = include_str!("../Dockerfile.template");
    let modified_dockerfile = modify_dockerfile(
        dockerfile_template,
        &args.base_image,
        project_dir,
        entrypoint,
        port,
    );

    // Write modified Dockerfile to temp directory
    let dockerfile_path = temp_dir.join("Dockerfile");
    fs::write(&dockerfile_path, modified_dockerfile)?;
    println!("Created Dockerfile: {}", dockerfile_path.display());

    // Build Docker image
    println!("\nBuilding Docker image...");
    let build_status = Command::new("docker")
        .args(["build", "-f", "./Dockerfile", "-t", image_name, "."])
        .current_dir(&temp_dir)
        .status()?;

    if !build_status.success() {
        return Err("Docker build failed".into());
    }

    println!("\n✓ Docker image built successfully: {}", image_name);

    // Push Docker image if requested
    if args.push {
        println!("\nPushing Docker image...");
        let push_status = Command::new("docker").args(["push", image_name]).status()?;

        if !push_status.success() {
            return Err("Docker push failed".into());
        }

        println!("✓ Docker image pushed successfully");
    }

    // Generate Helm chart
    println!("\n=== Generating Helm Chart ===");
    let (service_name, version) = read_service_info_from_pyproject(project_home)?;
    println!("Service name from pyproject.toml: {}", service_name);
    println!("Version from pyproject.toml: {}", version);

    let chart_dir = temp_dir.join(&service_name);
    let charts_template_dir = PathBuf::from("charts");

    if !charts_template_dir.exists() {
        return Err(format!(
            "Charts template directory does not exist: {}",
            charts_template_dir.display()
        )
        .into());
    }

    println!(
        "Copying charts template from {} to {}",
        charts_template_dir.display(),
        chart_dir.display()
    );
    copy_and_replace_charts(
        &charts_template_dir,
        &chart_dir,
        &service_name,
        &version,
        port,
        image_name,
    )?;

    // Run helm lint
    println!("\nRunning helm lint...");
    let lint_status = Command::new("helm")
        .args(["lint", chart_dir.to_str().unwrap()])
        .status()?;

    if !lint_status.success() {
        return Err("Helm lint failed".into());
    }

    println!("✓ Helm lint passed");

    // Run helm package
    println!("\nRunning helm package...");
    let package_status = Command::new("helm")
        .args(["package", chart_dir.to_str().unwrap()])
        .current_dir(&temp_dir)
        .status()?;

    if !package_status.success() {
        return Err("Helm package failed".into());
    }

    // Find the generated chart file
    let chart_file_name = format!("{}-{}.tgz", service_name, version);
    let chart_file_path = temp_dir.join(&chart_file_name);

    if chart_file_path.exists() {
        println!(
            "✓ Helm chart packaged successfully: {}",
            chart_file_path.display()
        );
        println!("\nGenerated Helm chart: {}", chart_file_name);
    } else {
        return Err(format!("Helm chart file not found: {}", chart_file_path.display()).into());
    }

    println!("\nTemporary directory: {}", temp_dir.display());
    println!("(Note: Temporary directory is left behind for inspection)");

    Ok(())
}

fn prompt(message: &str) -> Result<String, io::Error> {
    print!("{}: ", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn modify_dockerfile(
    template: &str,
    base_image: &str,
    project_dir: &str,
    entrypoint: &str,
    port: u16,
) -> String {
    template
        .replace("{BASE_IMAGE}", base_image)
        .replace("{PROJECT_DIR}", project_dir)
        .replace("{PORT}", &port.to_string())
        .replace("{ENTRYPOINT}", entrypoint)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        // Skip .venv directories
        if file_name == ".venv" {
            continue;
        }

        let dest_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

fn read_name_from_pyproject(project_home: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let pyproject_path = project_home.join("pyproject.toml");

    if !pyproject_path.exists() {
        return Err(format!("pyproject.toml not found in: {}", project_home.display()).into());
    }

    let content = fs::read_to_string(&pyproject_path)?;
    let value: Value = toml::from_str(&content)?;

    // Extract project name
    let name = value
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("Missing 'project.name' in pyproject.toml")?
        .to_string();

    Ok(name)
}

fn read_service_info_from_pyproject(
    project_home: &Path,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let pyproject_path = project_home.join("pyproject.toml");

    if !pyproject_path.exists() {
        return Err(format!("pyproject.toml not found in: {}", project_home.display()).into());
    }

    let content = fs::read_to_string(&pyproject_path)?;
    let value: Value = toml::from_str(&content)?;

    // Extract project name
    let name = value
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("Missing 'project.name' in pyproject.toml")?
        .to_string();

    // Extract version
    let version = value
        .get("project")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project.version' in pyproject.toml")?
        .to_string();

    Ok((name, version))
}

fn copy_and_replace_charts(
    src: &Path,
    dst: &Path,
    service_name: &str,
    version: &str,
    port: u16,
    image_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dst.join(&file_name);

        if path.is_dir() {
            copy_and_replace_charts(&path, &dest_path, service_name, version, port, image_name)?;
        } else {
            // Read file content
            let content = fs::read_to_string(&path)?;

            // Replace placeholders
            let modified_content = content
                .replace("{SERVICE_NAME}", service_name)
                .replace("{VERSION}", version)
                .replace("{PORT}", &port.to_string())
                .replace("{IMAGE_NAME}", image_name);

            // Write modified content
            fs::write(&dest_path, modified_content)?;
        }
    }

    Ok(())
}
