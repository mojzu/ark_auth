FROM sso/build-test:latest

COPY .devcontainer/build/scripts/wait-for-it.sh /wait-for-it.sh
RUN chmod +x /wait-for-it.sh

ENTRYPOINT ["npm", "run", "protractor", "sso_test/tmp/conf.docker.js"]
