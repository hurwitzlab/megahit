TAG = hurwitzlab/megahit:1.2.9

img:
	docker build --tag=$(TAG) .

run:
	docker run --rm $(TAG)

shell:
	docker run --rm -it $(TAG) bash

push: 
	docker push $(TAG)
