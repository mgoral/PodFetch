add-apt-repository --remove ppa:ubuntu-toolchain-r/test -y
apt update
env ACCEPT_EULA=Y apt upgrade
apt-get purge gcc-5 gcc-5-base gcc-6-base
apt install gcc-multilib

apt clean && apt -o DPkg::Options::='--force-confnew'  --assume-yes install gcc libssl-dev gcc-multilib musl musl-dev musl-tools