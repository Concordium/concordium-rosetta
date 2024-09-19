pipeline {
    agent any
    stages {
        stage('build') {
            agent { label 'mac' }
            steps {
                sh '''\
                    # Set up Rust toolchain.
                    rustup default 1.73 # TODO parameterize

                    # Build binary and run it to get version.
                    version="$(cargo run --release -- --version | awk '{print $2}')"

                    # Extract binary and append version to name.
                    mkdir ./out
                    cp ./target/release/concordium-rosetta ./out/concordium-rosetta_${version}
                '''.stripIndent()
                stash includes: 'out/', name: 'target'
            }
        }
        stage('push') {
            steps {
                unstash 'target' // transfers './out'.
                sh '''\
                    # Push binary to S3.
                    aws s3 cp \
                        ./out/concordium-rosetta* \
                        s3://distribution.concordium.software/tools/macos/ \
                        --grants=read=uri=http://acs.amazonaws.com/groups/global/AllUsers
                '''.stripIndent()
            }
        }
    }
}
