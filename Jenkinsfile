pipeline {
    agent any
    environment {
        image_repo = "concordium/rosetta"
        image_name = "${image_repo}:${image_tag}"
    }
    stages {
        stage('dockerhub-login') {
            environment {
                // Defines 'CRED_USR' and 'CRED_PSW'
                // (see 'https://www.jenkins.io/doc/book/pipeline/jenkinsfile/#handling-credentials').
                CRED = credentials('jenkins-dockerhub')
            }
            steps {
                sh 'docker login --username "${CRED_USR}" --password "${CRED_PSW}"'
            }
        }
        stage('build-push-docker') {
            steps {
                sh '''\
                    docker build \
                      --build-arg build_image="${build_image}" \
                      --label build_image="${build_image}" \
                      --build-arg base_image="${base_image}" \
                      --label base_image="${base_image}" \
                      --tag="${image_name}" \
                      --pull \
                      .
                    docker push "${image_name}"
                '''.stripIndent()
            }
        }
        stage('build-push-debian') {
            steps {
                sh '''\
                    # Building image from "build" stage in the docker file.
                    # It should be entirely cached.
                    docker build \
                        -t build \
                        --target=build \
                        --build-arg=build_image="${build_image}" \
                        --build-arg=base_image="${base_image}" \
                        --pull \
                        .
                    # Extract debian package from docker image into './out'.
                    # The file will have owner 'root' because docker volumes cannot be mounted as anything else
                    # (see 'https://github.com/moby/moby/issues/2259').
                    mkdir -p ./out
                    docker run --rm --volume="$(pwd)/out:/out" build
                    aws s3 cp \
                        ./out/concordium-rosetta*.deb \
                        s3://distribution.concordium.software/tools/linux/ \
                        --grants=read=uri=http://acs.amazonaws.com/groups/global/AllUsers
                '''.stripIndent()
            }
        }
    }
}
