use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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

    /// Python version to use
    #[arg(long, default_value = "3.14")]
    python_version: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Args::parse();

    // Prompt for missing required arguments
    if args.name.is_none() {
        args.name = Some(prompt("Project name")?);
    }

    if args.project_home.is_none() {
        let path_str = prompt("Project home path")?;
        args.project_home = Some(PathBuf::from(path_str));
    }
    let project_dir = args
        .project_home
        .as_ref()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

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
    println!("Python version: {}", args.python_version);
    println!("Push: {}", args.push);
    if let Some(registry) = &args.registry {
        println!("Registry: {}", registry);
    }
    println!("=====================\n");

    // Validate project home exists
    if !project_home.exists() {
        return Err(format!("Project home does not exist: {}", project_home.display()).into());
    }

    // Create temporary directory
    let temp_dir = std::env::temp_dir().join(format!("servicemaker-{}", name));
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
        &args.python_version,
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
    python_version: &str,
) -> String {
    let mut result = template.to_string();
    result = result.replacen("{}", base_image, 1);
    result = result.replacen("{}", project_dir, 1);
    result = result.replacen("{}", project_dir, 1);
    result = result.replacen("{}", project_dir, 1);
    result = result.replacen("{}", project_dir, 1);
    result = result.replacen("{}", python_version, 1);
    result = result.replacen("{}", &port.to_string(), 1);
    result = result.replacen("{}", entrypoint, 1);
    result
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
