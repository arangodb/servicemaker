use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;

// Default base images
const DEFAULT_PYTHON_BASE_IMAGE: &str = "arangodb/py12base:latest";
const DEFAULT_NODEJS_BASE_IMAGE: &str = "arangodb/node22base:latest";

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
    ChartFile {
        path: "templates/service-account.yaml",
        content: include_str!("../charts/templates/service-account.yaml"),
    },
    ChartFile {
        path: "templates/token-permissions.yaml",
        content: include_str!("../charts/templates/token-permissions.yaml"),
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
        path: "check-base-dependencies.js",
        content: include_str!("../scripts/check-base-dependencies.js"),
    },
    ScriptFile {
        path: "zipper.sh",
        content: include_str!("../scripts/zipper.sh"),
    },
    ScriptFile {
        path: "nvidia_lib_path.sh",
        content: include_str!("../baseimages/scripts/nvidia_lib_path.sh"),
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
    #[arg(long)]
    base_image: Option<String>,

    /// Exposed port number
    #[arg(long)]
    port: Option<u16>,

    /// Docker image name to push
    #[arg(long)]
    image_name: Option<String>,

    /// Whether to push the image
    #[arg(long, default_value = "false")]
    push: bool,

    /// Name of the entrypoint script to run (relative to project home)
    /// For Python: e.g., main.py
    /// For Node.js: e.g., index.js
    #[arg(long)]
    entrypoint: Option<String>,

    /// Whether to create a tar.gz file with project files and virtual environment changes
    #[arg(long, default_value = "false")]
    make_tar_gz: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Args::parse();

    // Track if base_image was explicitly set by user
    let base_image_explicitly_set = args.base_image.is_some();

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

    // Detect project type: "python" or "nodejs"
    let project_type = detect_project_type(project_home)?;
    println!("Detected project type: {}", project_type);

    // Handle project type-specific configuration
    match project_type.as_str() {
        "python" => {
            // Python project: requires pyproject.toml
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

            // Set default base image for Python if not explicitly set
            if !base_image_explicitly_set {
                args.base_image = Some(DEFAULT_PYTHON_BASE_IMAGE.to_string());
            }
        }
        "nodejs" => {
            // Node.js project: requires package.json (no services.json or manifest.json)
            // Try to get name from package.json if not provided
            if args.name.is_none()
                && let Ok(name) = read_name_from_package_json(project_home)
            {
                args.name = Some(name);
            }

            // Try to auto-detect entrypoint from package.json "main" field or "start" script
            if args.entrypoint.is_none() {
                if let Ok(Some(entrypoint)) = detect_nodejs_entrypoint(project_home) {
                    args.entrypoint = Some(entrypoint);
                } else {
                    args.entrypoint = Some("index.js".to_string());
                }
            }

            // Set default base image for Node.js if not explicitly set
            if !base_image_explicitly_set {
                args.base_image = Some(DEFAULT_NODEJS_BASE_IMAGE.to_string());
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
    let base_image = args.base_image.as_ref().unwrap();

    println!("\n=== Configuration ===");
    println!("Project name: {}", name);
    println!("Project type: {}", project_type);
    println!("Project home: {}", project_home.display());
    println!("Project directory name: {}", initial_project_dir);
    println!("Base image: {}", base_image);
    println!("Port: {}", port);
    println!("Image name: {}", image_name);
    if let Some(ref entrypoint) = args.entrypoint {
        println!("Entrypoint: {}", entrypoint);
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

    // Copy project directory to temp directory
    // Both Python and Node.js projects are copied directly without any special handling
    let project_dest = temp_dir.join(initial_project_dir);
    println!(
        "Copying project from {} to {}",
        project_home.display(),
        project_dest.display()
    );
    copy_dir_recursive(project_home, &project_dest)?;
    let project_dir = initial_project_dir.to_string();

    // Read environment variables from .env.example if it exists
    let env_vars = read_env_example(project_home)?;
    if !env_vars.is_empty() {
        println!("Found {} environment variable(s) in .env.example", env_vars.len());
    }

    // Choose Dockerfile template and modify based on project type
    let modified_dockerfile = match project_type.as_str() {
        "python" => {
            // Python project: use Python Dockerfile template
            let python_version = extract_python_version(base_image);
            let entrypoint = args.entrypoint.as_ref().unwrap();
            let dockerfile_template = include_str!("../Dockerfile.template");
            modify_dockerfile_python(
                dockerfile_template,
                base_image,
                &project_dir,
                entrypoint,
                port,
                &python_version,
                &env_vars,
            )
        }
        "nodejs" => {
            // Node.js project: use Node.js Dockerfile template
            let entrypoint = args.entrypoint.as_ref().unwrap();
            let dockerfile_template = include_str!("../Dockerfile.nodejs.template");
            modify_dockerfile_nodejs(
                dockerfile_template,
                base_image,
                &project_dir,
                entrypoint,
                port,
                &env_vars,
            )
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
            // Extract service name and version from pyproject.toml
            let (name, ver) = read_service_info_from_pyproject(project_home)?;
            println!("Service name from pyproject.toml: {}", name);
            println!("Version from pyproject.toml: {}", ver);
            (name, ver)
        }
        "nodejs" => {
            // Extract service name and version from package.json
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
    if let Some(py_pos) = base_image.find("py") {
        let after_py = &base_image[py_pos + 2..];
        if let Some(end_pos) = after_py.find(|c: char| !c.is_ascii_digit()) {
            let version_digits = &after_py[..end_pos];
            if !version_digits.is_empty() {
                // Convert "12" -> "3.12", etc.
                return format!("3.{}", version_digits);
            }
        } else if !after_py.is_empty() && after_py.chars().all(|c| c.is_ascii_digit()) {
            // Handle case where version digits extend to end of string
            return format!("3.{}", after_py);
        }
    }
    // Default fallback if pattern not found
    "3.12".to_string()
}

fn modify_dockerfile_python(
    template: &str,
    base_image: &str,
    project_dir: &str,
    entrypoint: &str,
    port: u16,
    python_version: &str,
    env_vars: &[(String, String)],
) -> String {
    let mut result = template
        .replace("{BASE_IMAGE}", base_image)
        .replace("{PROJECT_DIR}", project_dir)
        .replace("{PORT}", &port.to_string())
        .replace("{ENTRYPOINT}", entrypoint)
        .replace("{PYTHON_VERSION}", python_version);
    
    // Add environment variables if any
    if !env_vars.is_empty() {
        let env_lines: Vec<String> = env_vars
            .iter()
            .map(|(key, value)| format!("ENV {}={}", key, value))
            .collect();
        let env_block = format!("\n{}", env_lines.join("\n"));
        
        // Insert after WORKDIR line
        if let Some(pos) = result.find("WORKDIR")
            && let Some(newline_pos) = result[pos..].find('\n') {
            let insert_pos = pos + newline_pos + 1;
            result.insert_str(insert_pos, &env_block);
        }
    }
    
    result
}

/// Modify Node.js Dockerfile template with project-specific values
/// Sets up NODE_PATH to resolve from project node_modules first, then base node_modules
fn modify_dockerfile_nodejs(
    template: &str,
    base_image: &str,
    project_dir: &str,
    entrypoint: &str,
    port: u16,
    env_vars: &[(String, String)],
) -> String {
    // Node.js app structure:
    // - COPY copies the project directory directly
    // - WORKDIR is /project/{project-dir}
    // - node_modules is in /project/{project-dir}/node_modules
    // - NODE_PATH allows resolving from project node_modules first, then base node_modules
    let node_path = format!("/project/{}/node_modules:/home/user/node_modules", project_dir);
    
    let mut result = template
        .replace("{BASE_IMAGE}", base_image)
        .replace("{PROJECT_DIR}", project_dir)
        .replace("{WORKDIR}", project_dir)
        .replace("{ENTRYPOINT}", entrypoint)
        .replace("{PORT}", &port.to_string())
        .replace("{NODE_PATH}", &node_path);
    
    // Add environment variables if any
    if !env_vars.is_empty() {
        let env_lines: Vec<String> = env_vars
            .iter()
            .map(|(key, value)| format!("ENV {}={}", key, value))
            .collect();
        let env_block = format!("\n{}", env_lines.join("\n"));
        
        // Insert after NODE_PATH ENV line
        if let Some(pos) = result.find("ENV NODE_PATH")
            && let Some(newline_pos) = result[pos..].find('\n') {
            let insert_pos = pos + newline_pos + 1;
            result.insert_str(insert_pos, &env_block);
        }
    }
    
    result
}

/// Read environment variables from .env.example file
/// Parses KEY=VALUE format and handles quoted values
fn read_env_example(project_home: &Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let env_example_path = project_home.join(".env.example");
    
    if !env_example_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&env_example_path)?;
    let mut env_vars = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse KEY=VALUE format
        if let Some(equal_pos) = line.find('=') {
            let key = line[..equal_pos].trim().to_string();
            let mut value = line[equal_pos + 1..].trim().to_string();
            
            // Remove quotes if present (handles both single and double quotes)
            if (value.starts_with('"') && value.ends_with('"')) 
                || (value.starts_with('\'') && value.ends_with('\'')) {
                value = value[1..value.len() - 1].to_string();
            }
            
            // Skip if key is empty
            if !key.is_empty() {
                // If value contains spaces or special characters, quote it for Docker ENV
                let final_value = if value.contains(' ') || value.contains('$') || value.contains('\\') {
                    format!("\"{}\"", value.replace('"', "\\\""))
                } else {
                    value
                };
                env_vars.push((key, final_value));
            }
        }
    }
    
    Ok(env_vars)
}

/// Detect Node.js entrypoint from package.json
/// Checks "main" field first, then "start" script
fn detect_nodejs_entrypoint(project_home: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let package_json_path = project_home.join("package.json");
    
    if !package_json_path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&package_json_path)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;
    
    // Try to get from "main" field
    if let Some(main) = value.get("main").and_then(|m| m.as_str()) {
        return Ok(Some(main.to_string()));
    }
    
    // Try to extract from "start" script
    if let Some(scripts) = value.get("scripts")
        && let Some(start) = scripts.get("start").and_then(|s| s.as_str()) {
        // Extract the script name from "node index.js" or "node app.js"
        if let Some(script_name) = start.strip_prefix("node ") {
            return Ok(Some(script_name.trim().to_string()));
        }
    }
    
    Ok(None)
}

/// Detect project type: "python" or "nodejs"
/// Python: has pyproject.toml
/// Node.js: has package.json (and no services.json or manifest.json)
fn detect_project_type(project_home: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let pyproject = project_home.join("pyproject.toml");
    let package_json = project_home.join("package.json");
    let services_json = project_home.join("services.json");
    let manifest_json = project_home.join("manifest.json");

    if pyproject.exists() {
        // Python project detected
        Ok("python".to_string())
    } else if package_json.exists() {
        // Node.js project: must not have services.json or manifest.json (those are not supported)
        if services_json.exists() || manifest_json.exists() {
            return Err(format!(
                "Node.js projects with services.json or manifest.json are not supported. \
                This service only supports Python projects and simple Node.js projects with package.json only. \
                Found in: {}",
                project_home.display()
            ).into());
        }
        // Simple Node.js project
        Ok("nodejs".to_string())
    } else {
        Err(format!(
            "Could not detect project type. Expected pyproject.toml (Python) or package.json (Node.js) in: {}",
            project_home.display()
        )
        .into())
    }
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
