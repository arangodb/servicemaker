use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;

// Embedded chart files
struct ChartFile {
    path: &'static str,
    content: &'static str,
}

const CHART_FILES: &[ChartFile] = &[
    ChartFile {
        path: "Chart.yaml",
        content: include_str!("../charts/Chart.yaml"),
    },
    ChartFile {
        path: "values.yaml",
        content: include_str!("../charts/values.yaml"),
    },
    ChartFile {
        path: "templates/_helpers.tpl",
        content: include_str!("../charts/templates/_helpers.tpl"),
    },
    ChartFile {
        path: "templates/deployment.yaml",
        content: include_str!("../charts/templates/deployment.yaml"),
    },
    ChartFile {
        path: "templates/route.yaml",
        content: include_str!("../charts/templates/route.yaml"),
    },
    ChartFile {
        path: "templates/service.yaml",
        content: include_str!("../charts/templates/service.yaml"),
    },
];

// Embedded script files
struct ScriptFile {
    path: &'static str,
    content: &'static str,
}

const SCRIPT_FILES: &[ScriptFile] = &[
    ScriptFile {
        path: "prepareproject.sh",
        content: include_str!("../scripts/prepareproject.sh"),
    },
    ScriptFile {
        path: "prepareproject-nodejs.sh",
        content: include_str!("../scripts/prepareproject-nodejs.sh"),
    },
    ScriptFile {
        path: "zipper.sh",
        content: include_str!("../scripts/zipper.sh"),
    },
];

/// A tool to wrap Python and Node.js projects as Docker services
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the project
    #[arg(long)]
    name: Option<String>,

    /// Path to the folder containing the project
    #[arg(long)]
    project_home: Option<PathBuf>,

    /// Base Docker image
    #[arg(long, default_value = "arangodb/py13base:latest")]
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

    /// Name of the Python script to run (relative to project home)
    #[arg(long)]
    entrypoint: Option<String>,

    /// Whether to create a tar.gz file with project files and virtual environment changes
    #[arg(long, default_value = "false")]
    make_tar_gz: bool,

    /// Mount path for the service (required for Foxx services, e.g., /itz)
    #[arg(long)]
    mount_path: Option<String>,
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

    // Detect project type
    let project_type = detect_project_type(project_home)?;
    println!("Detected project type: {}", project_type);

    // Handle project type-specific configuration
    match project_type.as_str() {
        "python" => {
            // Try to get name from pyproject.toml if not provided on command line
            if args.name.is_none()
                && let Ok(name) = read_name_from_pyproject(project_home)
            {
                args.name = Some(name);
            }
            
            // Try to auto-detect entrypoint if exactly one .py file exists
            if args.entrypoint.is_none()
                && let Ok(Some(py_file)) = find_single_py_file(project_home)
            {
                args.entrypoint = Some(py_file);
            }

            // Prompt for entrypoint if still not set
            if args.entrypoint.is_none() {
                args.entrypoint = Some(prompt("Python entrypoint script (e.g., main.py)")?);
            }
        }
        "foxx" => {
            // Multi-service structure (has services.json)
            // Try to get name from package.json if not provided
            if args.name.is_none()
                && let Ok(name) = read_name_from_package_json(project_home)
            {
                args.name = Some(name);
            }

            // Set default base image for Node.js if not specified
            if args.base_image == "arangodb/py13base:latest" {
                args.base_image = "arangodb/node22base:latest".to_string();
            }
        }
        "foxx-service" | "nodejs" => {
            // Single service directory - will create wrapper structure
            let service_name = project_home.file_name().unwrap().to_str().unwrap();

            // Try to get name from package.json if not provided
            if args.name.is_none() {
                if let Ok(name) = read_name_from_package_json(project_home) {
                    args.name = Some(name);
                } else {
                    args.name = Some(service_name.to_string());
                }
            }

            // Set default base image for Node.js if not specified
            if args.base_image == "arangodb/py13base:latest" {
                args.base_image = "arangodb/node22base:latest".to_string();
            }

            // Prompt for mount path if not provided (for foxx-service)
            if project_type == "foxx-service" && args.mount_path.is_none() {
                let default_mount = format!("/{}", service_name.to_lowercase());
                let mount_input = prompt(&format!("Mount path (default: {})", default_mount))?;
                args.mount_path = Some(if mount_input.is_empty() {
                    default_mount
                } else {
                    mount_input
                });
            }
        }
        _ => {
            return Err(format!("Unsupported project type: {}", project_type).into());
        }
    }

    // Prompt for name if still not set
    if args.name.is_none() {
        args.name = Some(prompt("Project name")?);
    }

    let initial_project_dir = project_home.file_name().unwrap().to_str().unwrap();

    if args.port.is_none() {
        let port_str = prompt("Exposed port number")?;
        args.port = Some(port_str.parse().expect("Invalid port number"));
    }

    if args.image_name.is_none() {
        args.image_name = Some(prompt("Docker image name")?);
    }

    let name = args.name.as_ref().unwrap();
    let project_home = args.project_home.as_ref().unwrap();
    let image_name = args.image_name.as_ref().unwrap();
    let port = args.port.unwrap();

    println!("\n=== Configuration ===");
    println!("Project name: {}", name);
    println!("Project type: {}", project_type);
    println!("Project home: {}", project_home.display());
    println!("Project directory name: {}", initial_project_dir);
    println!("Base image: {}", args.base_image);
    println!("Port: {}", port);
    println!("Image name: {}", image_name);
    if let Some(ref entrypoint) = args.entrypoint {
        println!("Entrypoint: {}", entrypoint);
    }
    if let Some(ref mount_path) = args.mount_path {
        println!("Mount path: {}", mount_path);
    }
    println!("Push: {}", args.push);
    println!("Make tar.gz: {}", args.make_tar_gz);
    println!("=====================\n");

    // Create temporary directory
    let temp_dir =
        std::env::current_dir()?.join(format!("servicemaker-{}-{}", name, std::process::id()));
    println!("Creating temporary directory: {}", temp_dir.display());

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // Copy scripts to temp directory with executable permissions
    copy_scripts_to_temp(&temp_dir)?;

    // Handle Node.js single service directory differently - create wrapper structure
    let project_dir = if project_type == "foxx-service" {
        let service_name = initial_project_dir;

        // Create wrapper directory
        let wrapper_dir = temp_dir.join("wrapper");
        fs::create_dir_all(&wrapper_dir)?;

        // Copy service directory into wrapper
        let service_dest = wrapper_dir.join(service_name);
        println!(
            "Creating wrapper structure: copying {} to wrapper/{}",
            project_home.display(),
            service_name
        );
        copy_dir_recursive(project_home, &service_dest)?;

        // Copy package.json from service to wrapper root for dependency installation
        let service_package_json = service_dest.join("package.json");
        if service_package_json.exists() {
            let wrapper_package_json = wrapper_dir.join("package.json");
            fs::copy(&service_package_json, &wrapper_package_json)?;
            println!("Copied package.json to wrapper root for dependency installation");
        }

        // Generate services.json with mount path
        let mount_path = args
            .mount_path
            .as_deref()
            .ok_or("Mount path is required for foxx-service")?;
        let services_json_content = generate_services_json(service_name, mount_path);
        let services_json_path = wrapper_dir.join("services.json");
        fs::write(&services_json_path, services_json_content)?;
        println!("Generated services.json: {}", services_json_path.display());

        "wrapper".to_string()
    } else {
        // Normal case - copy project directory as-is
        let project_dest = temp_dir.join(initial_project_dir);
        println!(
            "Copying project from {} to {}",
            project_home.display(),
            project_dest.display()
        );
        copy_dir_recursive(project_home, &project_dest)?;
        initial_project_dir.to_string()
    };

    // Choose Dockerfile template and modify based on project type
    let modified_dockerfile = match project_type.as_str() {
        "python" => {
            let python_version = extract_python_version(&args.base_image);
            let entrypoint = args.entrypoint.as_ref().unwrap();
            let dockerfile_template = include_str!("../Dockerfile.template");
            modify_dockerfile_python(
                dockerfile_template,
                &args.base_image,
                &project_dir,
                entrypoint,
                port,
                &python_version,
            )
        }
        "foxx" | "foxx-service" | "nodejs" => {
            let dockerfile_template = include_str!("../Dockerfile.nodejs.template");
            modify_dockerfile_nodejs(dockerfile_template, &args.base_image, &project_dir, port)
        }
        _ => return Err("Unsupported project type".into()),
    };

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

    // Create tar.gz file if requested
    if args.make_tar_gz {
        println!("\n=== Creating project.tar.gz ===");

        // Run container in detached mode to get container ID
        let container_output = Command::new("docker")
            .args([
                "run",
                "-d",
                "--entrypoint",
                "bash",
                image_name,
                "-c",
                &format!("/scripts/zipper.sh {}", project_dir),
            ])
            .output()?;

        if !container_output.status.success() {
            return Err(format!(
                "Failed to start Docker container: {}",
                String::from_utf8_lossy(&container_output.stderr)
            )
            .into());
        }

        let container_id = String::from_utf8(container_output.stdout)?
            .trim()
            .to_string();
        println!("Started container: {}", container_id);

        // Wait for container to finish
        println!("Waiting for container to finish...");
        let wait_status = Command::new("docker")
            .args(["wait", &container_id])
            .status()?;

        if !wait_status.success() {
            return Err("Failed to wait for container".into());
        }

        // Check exit code of the container
        let exit_code_output = Command::new("docker")
            .args(["inspect", "-f", "{{.State.ExitCode}}", &container_id])
            .output()?;

        if !exit_code_output.status.success() {
            return Err("Failed to inspect container exit code".into());
        }

        let exit_code = String::from_utf8(exit_code_output.stdout)?
            .trim()
            .parse::<i32>()?;

        if exit_code != 0 {
            return Err(format!("Container exited with code: {}", exit_code).into());
        }

        // Copy file from container to temp directory
        let tar_file_path = temp_dir.join("project.tar.gz");
        println!("Copying project.tar.gz from container...");
        let copy_status = Command::new("docker")
            .args([
                "cp",
                &format!("{}:/tmp/project.tar.gz", container_id),
                tar_file_path.to_str().unwrap(),
            ])
            .status()?;

        if !copy_status.success() {
            return Err("Failed to copy project.tar.gz from container".into());
        }

        // Remove the container
        println!("Removing container...");
        let rm_status = Command::new("docker")
            .args(["rm", &container_id])
            .status()?;

        if !rm_status.success() {
            return Err("Failed to remove container".into());
        }

        if tar_file_path.exists() {
            println!(
                "✓ project.tar.gz created successfully: {}",
                tar_file_path.display()
            );
        } else {
            return Err(format!("project.tar.gz not found at: {}", tar_file_path.display()).into());
        }
    }

    // Generate Helm chart
    println!("\n=== Generating Helm Chart ===");
    let (service_name, version) = match project_type.as_str() {
        "python" => {
            let (name, ver) = read_service_info_from_pyproject(project_home)?;
            println!("Service name from pyproject.toml: {}", name);
            println!("Version from pyproject.toml: {}", ver);
            (name, ver)
        }
        "foxx" | "foxx-service" | "nodejs" => {
            let (name, ver) = read_service_info_from_package_json(project_home)?;
            println!("Service name from package.json: {}", name);
            println!("Version from package.json: {}", ver);
            (name, ver)
        }
        _ => return Err("Unsupported project type for Helm chart generation".into()),
    };

    let chart_dir = temp_dir.join(&service_name);

    println!("Generating charts template in {}", chart_dir.display());
    copy_and_replace_charts(&chart_dir, &service_name, &version, port, image_name)?;

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

fn extract_python_version(base_image: &str) -> String {
    // Extract Python version from base image name (e.g., "py13base" -> "3.13", "py12base" -> "3.12")
    if let Some(py_pos) = base_image.find("py") {
        let after_py = &base_image[py_pos + 2..];
        if let Some(end_pos) = after_py.find(|c: char| !c.is_ascii_digit()) {
            let version_digits = &after_py[..end_pos];
            if !version_digits.is_empty() {
                // Convert "13" -> "3.13", "12" -> "3.12", etc.
                return format!("3.{}", version_digits);
            }
        } else if !after_py.is_empty() && after_py.chars().all(|c| c.is_ascii_digit()) {
            // Handle case where version digits extend to end of string
            return format!("3.{}", after_py);
        }
    }
    // Default fallback if pattern not found
    "3.13".to_string()
}

fn modify_dockerfile_python(
    template: &str,
    base_image: &str,
    project_dir: &str,
    entrypoint: &str,
    port: u16,
    python_version: &str,
) -> String {
    template
        .replace("{BASE_IMAGE}", base_image)
        .replace("{PROJECT_DIR}", project_dir)
        .replace("{PORT}", &port.to_string())
        .replace("{ENTRYPOINT}", entrypoint)
        .replace("{PYTHON_VERSION}", python_version)
}

fn modify_dockerfile_nodejs(
    template: &str,
    base_image: &str,
    project_dir: &str,
    port: u16,
) -> String {
    template
        .replace("{BASE_IMAGE}", base_image)
        .replace("{PROJECT_DIR}", project_dir)
        .replace("{PORT}", &port.to_string())
}

fn detect_project_type(project_home: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let pyproject = project_home.join("pyproject.toml");
    let package_json = project_home.join("package.json");
    let services_json = project_home.join("services.json");

    if pyproject.exists() {
        Ok("python".to_string())
    } else if package_json.exists() && services_json.exists() {
        Ok("foxx".to_string())
    } else if package_json.exists() {
        // Single service directory - needs wrapper structure
        Ok("foxx-service".to_string())
    } else {
        Err(format!(
            "Could not detect project type. Expected pyproject.toml (Python) or package.json (Node.js) in: {}",
            project_home.display()
        )
        .into())
    }
}

fn generate_services_json(service_name: &str, mount_path: &str) -> String {
    format!(
        r#"[
    {{
        "mount": "{}",
        "basePath": "{}"
    }}
]"#,
        mount_path, service_name
    )
}

fn read_name_from_package_json(project_home: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let package_json_path = project_home.join("package.json");

    if !package_json_path.exists() {
        return Err(format!("package.json not found in: {}", project_home.display()).into());
    }

    let content = fs::read_to_string(&package_json_path)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;

    // Extract project name
    let name = value
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("Missing 'name' in package.json")?
        .to_string();

    Ok(name)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        // Skip .venv directories (Python) and node_modules (Node.js)
        if file_name == ".venv" || file_name == "node_modules" {
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

fn find_single_py_file(project_home: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut py_files = Vec::new();

    for entry in fs::read_dir(project_home)? {
        let entry = entry?;
        let path = entry.path();

        // Only check files (not directories) and only at the root level
        if path.is_file()
            && let Some(extension) = path.extension()
            && extension == "py"
            && let Some(file_name) = path.file_name()
            && let Some(name_str) = file_name.to_str()
        {
            py_files.push(name_str.to_string());
        }
    }

    // Return the filename if exactly one .py file is found
    if py_files.len() == 1 {
        Ok(Some(py_files[0].clone()))
    } else {
        Ok(None)
    }
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

fn read_service_info_from_package_json(
    project_home: &Path,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let package_json_path = project_home.join("package.json");

    if !package_json_path.exists() {
        return Err(format!("package.json not found in: {}", project_home.display()).into());
    }

    let content = fs::read_to_string(&package_json_path)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;

    // Extract project name
    let name = value
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("Missing 'name' in package.json")?
        .to_string();

    // Extract version (default to "1.0.0" if not present)
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();

    Ok((name, version))
}

fn copy_scripts_to_temp(temp_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let scripts_dir = temp_dir.join("scripts");
    fs::create_dir_all(&scripts_dir)?;

    // Process each embedded script file
    for script_file in SCRIPT_FILES {
        let dest_path = scripts_dir.join(script_file.path);

        // Write script content
        fs::write(&dest_path, script_file.content)?;

        // Set executable permissions (0o755 = rwxr-xr-x)
        let mut perms = fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms)?;
    }

    println!("Created scripts directory: {}", scripts_dir.display());
    Ok(())
}

fn copy_and_replace_charts(
    dst: &Path,
    service_name: &str,
    version: &str,
    port: u16,
    image_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    // Process each embedded chart file
    for chart_file in CHART_FILES {
        // Create the full destination path
        let dest_path = dst.join(chart_file.path);

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Replace placeholders in the embedded content
        let modified_content = chart_file
            .content
            .replace("{SERVICE_NAME}", service_name)
            .replace("{VERSION}", version)
            .replace("{PORT}", &port.to_string())
            .replace("{IMAGE_NAME}", image_name);

        // Write modified content
        fs::write(&dest_path, modified_content)?;
    }

    Ok(())
}
