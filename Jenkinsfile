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
                      .
                    docker push "${image_name}"
                '''.stripIndent()
            }
        }
        stage('build-push-debian') {
            steps {
                sh '''\
                    BUILD_IMAGE="${build_image}" ./build-deb.sh
                    aws s3 cp ./concordium-rosetta*.deb s3://distribution.concordium.software/tools/linux/
                '''.stripIndent()
            }
        }
    }
}
