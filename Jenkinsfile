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
        stage('build') {
            steps {
                sh '''\
                    docker build \
                      --build-arg build_image="${build_image}" \
                      --label build_image="${build_image}" \
                      --build-arg base_image="${base_image}" \
                      --label base_image="${base_image}" \
                      --tag="${image_name}" \
                      .
                '''.stripIndent()
            }
        }
        stage('push') {
            steps {
                sh 'docker push "${image_name}"'
            }
        }
    }
}
