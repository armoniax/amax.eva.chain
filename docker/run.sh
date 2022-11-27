NET=$1
TAG=$2

mkdir -p ~/.amax_eva/$NET/$TAG
cd ~/.amax_eva/$NET/$TAG
[ ! -f ".env" ] && echo "missing .env" && exit 1

source .env && mkdir -p $EVA_SHARE

docker-compose up -d