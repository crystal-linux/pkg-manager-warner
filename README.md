<p align="center">
  <a href="https://git.getcryst.al/crystal">
    <img src="https://git.getcryst.al/crystal/branding/raw/branch/main/logos/crystal-logo-minimal.png" alt="Logo" width="150" height="150">
  </a>
</p>
<p align="center"> 
<h2 align="center"> pkg manager warner </h2>
</p>
<p align="center">
<a href="https://discord.gg/yp4xpZeAgW"><img alt="Discord" src="https://img.shields.io/discord/825473796227858482?color=blue&label=Discord&logo=Discord&logoColor=white"?link=https://discord.gg/yp4xpZeAgW&link=https://discord.gg/yp4xpZeAgW> </p></a>
<p align="center"> a tool to warn users when using the wrong package manager </p>


## Usage:
| Action | shorthand flag | long flag |
| ------ | ------ | ------ |
| Show a help message | pkg-warner -h | pkg-warner help |
| Show the version | pkg-warner -v | pkg-warner version |
| initialize the database | ame -i | ame init |
| create all the scripts | pkg-warner -c | pkg-warner create-script |
| Show the warning for specified package manager | pkg-warner -w \<package-manager> | pkg-warner warning \<package-warner> |

## Installation for Non-Crystal systems:
If you're on an arch (deriviate), you can use the PKGBUILD in [packages/pkg-warner](https://git.getcryst.al/crystal/packages-x86_64/src/branch/master/pkg-warner)
If you're on a different distribution you can use these commands:
```sh
git clone https://git.getcryst.al/crystal/pkg-manager-warner.git
cd pkg-manager-warner
sudo make install
```