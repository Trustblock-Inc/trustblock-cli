#!/bin/bash

if [ -z "$(docker ps -f "name=app" -f "status=running" -q)" ]; then
    docker compose up -d

else
    docker compose exec app npx prisma migrate reset -f --skip-generate &&
        docker compose exec app npx prisma db push --skip-generate
fi

# Loop until the app is ready or we've tried 10 times
for i in {1..10}; do
    echo "Attempt $i to connect to the app..."

    # Try to connect to the app & add auditor
    if curl --output /dev/null --silent --head --fail http://localhost:3000; then

        source .env

        trustblock-internal add-user-db -d ./tests/test-data/auditor.json -e http://localhost:3000/api/user/auditor -m $MASTER_KEY

        API_KEY=$(mysql -h127.0.0.1 -uuser -ppass local -Bse "SELECT \`key\` FROM ApiKey;")

        trustblock clean && trustblock init -p $WALLET_KEY -a $API_KEY

        content_to_append="\nAUDIT_ENDPOINT=$AUDIT_ENDPOINT\nPROJECT_ENDPOINT=$PROJECT_ENDPOINT\nFORWARDER_ENDPOINT=$FORWARDER_ENDPOINT\nWEB3_STORAGE_API_ENDPOINT=$WEB3_STORAGE_API_ENDPOINT\nTB_CORE_ADDRESS=$TB_CORE_ADDRESS\nPDF_GENERATE_ENDPOINT=$PDF_GENERATE_ENDPOINT"

        # The file you want to append to
        file_path="$HOME/.trustblock/.env"

        # Append the content to the file
        echo -e "$content_to_append" >>"$file_path"

        break
    else
        echo "App is not ready yet, retrying in 5 seconds..."
        sleep 5
    fi
done
