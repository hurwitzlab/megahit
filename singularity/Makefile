SINGULARITY = /usr/bin/singularity
IMG_NAME = megahit-1.1.2.img
DEF_NAME = ubuntu.sh
SIZE = 512

run:
	sudo $(SINGULARITY) run $(IMG_NAME)

clean:
	rm -f $(IMG_NAME)

img: clean
	sudo $(SINGULARITY) create --size $(SIZE) $(IMG_NAME)
	sudo $(SINGULARITY) bootstrap $(IMG_NAME) $(DEF_NAME)

shell:
	sudo $(SINGULARITY) shell --writable -B $(shell pwd):/tmp $(IMG_NAME)

scp:
	scp $(IMG_NAME) stampede:/work/03137/kyclark/$(IMG_NAME)/stampede
