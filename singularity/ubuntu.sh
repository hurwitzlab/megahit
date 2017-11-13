BootStrap: debootstrap
OSVersion: trusty
MirrorURL: http://us.archive.ubuntu.com/ubuntu/

%environment
    PATH=/app/megahit/scripts:$PATH

%runscript
    exec /app/megahit/bin/megahit "$@"

%post
    apt-get update
    apt-get install -y locales git build-essential wget # curl libcurl4-openssl-dev libssl-dev python3 python3-pip
    locale-gen en_US.UTF-8
    #yum update -y

    #
    # Put everything into $APP_DIR
    #
    export APP_DIR=/app
    mkdir -p $APP_DIR
    cd $APP_DIR

    #
    # Stampede code
    #
    cd $APP_DIR
    #git clone https://github.com/hurwitzlab/megahit.git mash

    wget -O megahit.tgz https://github.com/voutcn/megahit/releases/download/v1.1.2/megahit_v1.1.2_LINUX_CPUONLY_x86_64-bin.tar.gz
    mkdir bin
    tar -xvf megahit.tgz -C bin --strip-components=1

    #
    # Mount points for TACC directories
    #
    mkdir /home1
    mkdir /scratch
    mkdir /work
