# Platform services

**Idea**: We want to make it as easy as possible for users of the platform to
deploy some service as a platform service, both on-prem and on AMP. 

## Overview

The crucial thing here is that it is **easy to use** and **convenient for
the customer**. As a benchmark: For our <= 3.12.* Foxx services one had
to use JavaScript, but one could use a **standard npm project**, zip it up
in one zipfile and use a single API call to deploy it to run on the coordinators
under some path! Since we remove this functionality in >= 4.0.0, we need
a replacement.

Being a "platform service" means that it is running in some pod, relatively
close to the database, with database access (given the necessary credentials)
and accessible via envoy over the normal platform endpoint under some path.

It is desirable that such services can be written in basically any language.

For deployment on AMP (our own managed service), we need to fulfill a lot of
requirements w.r.t. security like properly isolating such services from other
customer deployments, regular scanning for vulnerabilities and the like.

The basic approach is a combination of some "base Docker images" with a tool, 
which we can publish and users can run it on their machines (outside the
platform) that does the following, basically all in one call:

 - take "a standard project" and turns it into a Docker container image (derived
   from one of our base images)
 - pushes this container image to a configurable registry
 - creates a Helm chart with just a handful of configuration parameters
   (like for example the route)
 - create a zipfile which can simply be unzipped on top of the chosen 
   base image, so that the service can run in that base image

Here, it is crucial that the place where the zipfile is extracted
is **separate from all files in the base image**, because these files
must be **immutable**. For now, we put the files and additional dependencies
under `/project` and use permissions of the user `user`, which exists in
the base images already.

This approach offers a number of deployment possibilities for different
situations:

0. Customers can run the Docker image locally for testing and debugging.
1. Customers on-prem with k8s access can simply deploy the Helm chart and
   the service is automatically deployed as platform service. This can be
   limited with k8s RBAC (on-prem users gets access only to 
   `ArangoPlatformChart`+`ArangoPlatformService` which enforces the boundary
   of Operator RBAC)
2. Customers on-prem without k8s access can use the Gen-AI deployment API with
   a "user-defined" generic service that can take the name of the Docker image
   as configurable Helm value
   
   There are 2 flows: 
     - Admin - any Docker image, with Helm chart - CICD compliant, as 
       they can create in CI `ArangoPlatformChart` + `ArangoPlatformService`
       instead of using the installer, and
     - User - GenAI supervised, untrusted code deployment
3. On AMP, we can pre-scan our standard base images and create a deployment
   service, which allows users push it to our AMP directly, and scanning 
   is done on this level. This supports Helm Charts (OCI) & Images
4. Later on AMP, we can allow for arbitrary Docker images to be automatically
   pushed to our own Docker registry, scanned, and deployed as platform
   services via some API.
5. Short term, we can allow proof of concepts to be done on our own devstack,
   either via 1. or 2 (1 is ready with 1.3.2 Operator it is possible to pull
   it from anywhere).

We have to provide tooling for the following cases:

 - Python (highest prio for now because of networkx, cugraph and AI libs.
 - JavaScript (second highest prio, since node-Foxx is some way off)
 - "Pure" projects which somehow can compile to an executable, potentially
   with some "assets". This can include obfuscated Python processes.

Are others needed?

Questionable for me:

 - Java?

## Python

For Python, I propose to implement the above in the following way: Our tool
(sample implementation in the `servicemaker` 
[here](https://github.com/arangodb/servicemaker) ) defines a few base images
for various Python versions and potentially different selections of 
pre-installed libs. 

All of them start with the `debian:trixie-slim` image and install a user
`user` who has `uv` installed. Then they have a certain Python version
(for now maybe 3.12 and 3.13) preinstalled via `uv` and they create a
virtual env in the home directory of `user` called `the_venv`. They can
then preinstall any number of libraries we choose into that virtual
environment. This is easy to do via `uv pip install` once the virtual
environment is activated.

Then the tool takes the Python project (which should run on the selected
Python version and have all its dependencies declared in `pyproject.toml`),
and we build a derived Docker image by using `uv install -r pyproject.toml` 
**in the same virtual environment**. This has the effect to only install
those dependencies in addition, which are not already contained in the
virtual env in the base image.

We can the produce a zipfile in the following way: The base image can
include a file with a list of all files in the virtual environment together
with their SHA256 sum. This means that after successful creation of the
final Docker image, we can simply check that no files in the virtual env
have changed and which have been added. We can then move over the new files
to a parallel directory hierarchy under `/project/the_venv/...`. 

To run the project, we simply activate the virtual environment in 
`/home/user/the_venv` and additional set the `PYTHONPATH` to also find
stuff in `/project/the_venv/lib/python3.13/site-packages/` for the 
additional dependencies.

We can then create a single zipfile, which includes all new files as
well as the files of the project itself. This means that if one simply
extracts the zipfile in `/project` on top of the base Docker image, one
can reproduce the complete installation exactly.

This means one can either run the derived Docker image directly, or one
can implement a solution to take the zipfile, extract it in `/project`,
set a few environment variables and then run the code.

This approach combines:

 - ease of use for the user
 - very high compatibility with modern Python tooling and robustness
 - start from a number of well-defined base images (good for sharing layers
   and for pre-scanning)
 - ready-made Docker image for local testing and quick-and-dirty deployment
 - zipfile approach for easier scanning and future-proof deployment on AMP

If this works well, we can replicate the approach for other languages. It
seems conceivable that node-based approaches with modern JS tools work exactly
the same way.
