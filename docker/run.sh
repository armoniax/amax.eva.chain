NET=$1
TAG=$2

cd ~/.amax_eva/$NET/$TAG
source .env && mkdir -p $EVA_SHARE

docker-compose up -d