pipeline {
    agent any
    stages {
        stage('build') {
            agent { label 'mac' }
            steps {
                sh '''\
                    # Set up Rust toolchain.
                    rustup default 1.53

                    # Build binary and run it to get version.
                    version="$(cargo run --release -- --version | awk '{print $2}')"

                    # Push binary to S3.
                    aws s3 cp \
                        ./target/release/concordium-rosetta.exe \
                        "s3://distribution.concordium.software/tools/macos/concordium-rosetta_${version}.exe" \
                        --grants=read=uri=http://acs.amazonaws.com/groups/global/AllUsers
                '''.stripIndent()
                stash includes: './target/release/concordium-rosetta', name: 'target'
            }
        }
        stage('push') {
            steps {
                unstash 'target'
                sh '''\
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
