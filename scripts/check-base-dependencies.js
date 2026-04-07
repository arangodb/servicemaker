#!/usr/bin/env node
/**
 * Checks which dependencies from package.json are already satisfied by base node_modules.
 * Returns a list of packages that need to be installed (missing or incompatible versions).
 * 
 * This script ensures we don't duplicate packages that already exist in the base image.
 */

const fs = require('fs');
const path = require('path');
const semver = require('semver');

const BASE_NODE_MODULES = '/home/user/node_modules';
const PROJECT_PACKAGE_JSON = './package.json';

// Read project package.json
let projectPackageJson;
try {
    projectPackageJson = JSON.parse(fs.readFileSync(PROJECT_PACKAGE_JSON, 'utf8'));
} catch (error) {
    console.error(`Error reading ${PROJECT_PACKAGE_JSON}:`, error.message);
    process.exit(1);
}

// Get all dependencies (dependencies + devDependencies, but we only care about production)
const allDependencies = {
    ...(projectPackageJson.dependencies || {}),
    // Note: We're in production mode, but checking all for completeness
};

if (Object.keys(allDependencies).length === 0) {
    console.log('No dependencies found in package.json');
    process.exit(0);
}

// Check which packages need to be installed
const packagesToInstall = [];
const packagesFromBase = [];
const filteredDependencies = {};

for (const [packageName, versionRange] of Object.entries(allDependencies)) {
    const basePackagePath = path.join(BASE_NODE_MODULES, packageName);
    const basePackageJsonPath = path.join(basePackagePath, 'package.json');
    
    // Check if package exists in base node_modules
    if (fs.existsSync(basePackageJsonPath)) {
        try {
            const basePackageJson = JSON.parse(fs.readFileSync(basePackageJsonPath, 'utf8'));
            const baseVersion = basePackageJson.version;
            
            // Check if base version satisfies project requirement
            if (semver.satisfies(baseVersion, versionRange)) {
                packagesFromBase.push(`${packageName}@${baseVersion} (satisfies ${versionRange})`);
                // Package is available in base and version is compatible - skip installation
                continue;
            } else {
                // Package exists but version is incompatible - need to install project version
                packagesToInstall.push(`${packageName}@${versionRange}`);
                filteredDependencies[packageName] = versionRange;
            }
        } catch (error) {
            // Error reading base package.json - install to be safe
            console.warn(`Warning: Could not read base package.json for ${packageName}, will install`);
            packagesToInstall.push(`${packageName}@${versionRange}`);
            filteredDependencies[packageName] = versionRange;
        }
    } else {
        // Package doesn't exist in base - need to install
        packagesToInstall.push(`${packageName}@${versionRange}`);
        filteredDependencies[packageName] = versionRange;
    }
}

const output = {
    packagesToInstall: packagesToInstall,
    filteredDependencies: filteredDependencies,
    packagesFromBase: packagesFromBase.length,
    totalDependencies: Object.keys(allDependencies).length
};

process.stderr.write(`\n=== Dependency Analysis ===\n`);
process.stderr.write(`Total dependencies: ${Object.keys(allDependencies).length}\n`);
process.stderr.write(`From base node_modules: ${packagesFromBase.length}\n`);
process.stderr.write(`To install: ${packagesToInstall.length}\n`);

if (packagesFromBase.length > 0) {
    process.stderr.write(`\nPackages using base version:\n`);
    packagesFromBase.forEach(pkg => process.stderr.write(`  ✓ ${pkg}\n`));
}

if (packagesToInstall.length > 0) {
    process.stderr.write(`\nPackages to install:\n`);
    packagesToInstall.forEach(pkg => process.stderr.write(`  → ${pkg}\n`));
}

// Write JSON to stdout (shell script will capture this separately)
process.stdout.write(JSON.stringify(output));

