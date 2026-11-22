#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if [ -e .env ]; then
    echo "Le fichier .env existe déjà." >&2
    exit 1
fi
umask 077
set -C
{
    for key in POSTGRES_PASSWORD SUPERVISOR_PASSWORD OPERATOR_A_PASSWORD OPERATOR_B_PASSWORD; do
        secret=$(openssl rand -hex 24)
        printf '%s=%s\n' "$key" "$secret"
    done
    printf 'SPACE_ERP_PORT=8080\n'
} > .env
