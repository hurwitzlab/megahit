#!/bin/bash

#SBATCH -A iPlant-Collabs
#SBATCH -t 02:00:0
#SBATCH -N 1
#SBATCH -n 1
#SBATCH -J megahit
#SBATCH -p development
#SBATCH --mail-type BEGIN,END,FAIL
#SBATCH --mail-user kyclark@email.arizona.edu

set -u

./run-megahit.pl -d $SCRATCH/data/assembly/velvet-fa -o $SCRATCH/megahit-out --debug
