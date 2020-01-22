IMG="/work/05066/imicrobe/singularity/megahit-1.2.9.img"

if [[ ! -e "$IMG" ]]; then
    echo "Missing Singularity image \"$IMG\""
    exit 1
fi

singularity exec $IMG run_megahit -o "megahit-out" ${IN_DIR} ${MIN_COUNT} ${K_MIN} ${K_MAX} ${K_STEP} ${MIN_CONTIG_LEN}

echo "Comments to kyclark@email.arizona.edu"
