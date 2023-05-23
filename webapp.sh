#!/bin/bash

source .env

add_db() {
    trustblock-internal add-user-db -d ./tests/test-data/auditor.json -e http://localhost:3000/api/user/auditor -m "$MASTER_KEY"

    API_KEY=$(mysql -h127.0.0.1 -uuser -ppass local -Bse "SELECT \`key\` FROM ApiKey;")

    # CLI env path
    CLI_FOLDER="$HOME/.trustblock"

    rm -rf "$CLI_FOLDER" && mkdir "$CLI_FOLDER"

    content_to_append="\nWALLET_KEY=$WALLET_KEY\nAPI_KEY=$API_KEY\nAUDIT_ENDPOINT=$AUDIT_ENDPOINT\nPROJECT_DOMAIN_ENDPOINT=$PROJECT_DOMAIN_ENDPOINT\nFORWARDER_ENDPOINT=$FORWARDER_ENDPOINT\nWEB3_STORAGE_API_ENDPOINT=$WEB3_STORAGE_API_ENDPOINT\nTB_CORE_ADDRESS=$TB_CORE_ADDRESS\nPDF_GENERATE_ENDPOINT=$PDF_GENERATE_ENDPOINT"

    # Append the content to the .env file
    echo -e "$content_to_append" >>"$CLI_FOLDER/.env"
}

if [ -z "$(docker ps -f "name=app" -f "status=running" -q)" ]; then
    docker compose up -d

    # Loop until the app is ready or we've tried 10 times
    for i in {1..10}; do
        echo "Attempt $i to connect to the app..."

        # Try to connect to the app & add auditor
        if curl --output /dev/null --silent --head --fail http://localhost:3000; then
            add_db
            break
        else
            echo "App is not ready yet, retrying in 5 seconds..."
            sleep 5
        fi
    done

else
    docker compose exec -T app npx prisma migrate reset -f --skip-generate &&
        docker compose exec -T app npx prisma db push --skip-generate && add_db
fi
