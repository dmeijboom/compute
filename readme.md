# compute - Modular computer provisioning tool

## Documentation
See the [wiki](https://github.com/dmeijboom/compute/wiki).

## Introduction
For a while now I'm using [multipass](https://github.com/canonical/multipass) to manage multiple vm's on my Linux laptop.
Since I'm a freelancer this is very neat because for each one of my clients I'm running a different vm (because I need different tools).

At first I was using cloud-init to provision those vm's but that didn't work for me as for example I encountered a situation where I wanted to do something that depends on another module but that module was initialized after the other module so that didn't work..

After a while I stopped using it and provisioned them manually but that required too much time.
That's when I decided to create this project, a simple but modular computer provisioning tool.
Is it better than cloud-init? Definitely lot.. But it's way easier to use and covers my use-case.

## Features
Compute supports one or more config files which can use all of these core modules:

- Modules (only builtin/core modules work, you can't write user-modules currently)
- Install apt packages
- Configure apt repositories
- Download S3 files (for SSH/GPG keys for example)
- Templates

## How does it work?
When running `compute apply` it reads the configuration file and applies each change per module.
If nothing changed (for example because a package was already installed) it won't do anything for that change.

## Current stage
It mostly works but there are a lot of things which can be improved or are still not done:

- Unittests (I should've done that already but yeah I'm lazy..)
- User modules
- Atomic applies (currently if something fails you're system is basically in an unknown state)
- Proper installation and update mechanism for the core modules
- Bugs (configuring an apt repository doesn't work in almost all cases)
- Other commands such as:
    - edit; Edit templates on-the-fly
    - ls; List repos which contain compute config
    - history; Show history
    - rollback; Roll back to a specific version in the history
    - upload; Upload a file to one of the S3 buckets
    - make-vm; Create and provision a multipass VM based on a compute config file
