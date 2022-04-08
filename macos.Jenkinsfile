pipeline {
    agent any
    stages {
        stage('build') {
            agent { label 'mac' }
            steps {
                sh '''\
                    # Set up Rust toolchain and build binary.
                    rustup default 1.53
                    cargo build --release
                '''.stripIndent()
                stash includes: 'target/release/concordium-rosetta', name: 'target'
            }
        }
        stage('push') {
            steps {
                unstash 'target' // transfers './target/release/concordium-rosetta'.
                sh '''\
                    # Run binary to get version.
                    version="$(./target/release/concordium-rosetta --version | awk '{print $2}')"
                    # Push binary to S3.
                    aws s3 cp \
                        ./target/release/concordium-rosetta \
                        "s3://distribution.concordium.software/tools/macos/concordium-rosetta_${version}" \
                        --grants=read=uri=http://acs.amazonaws.com/groups/global/AllUsers
                '''.stripIndent()
            }
        }
    }
}
