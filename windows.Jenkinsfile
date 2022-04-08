pipeline {
    agent { label 'windows' }
    stages {
        stage('build') {
            steps {
                sh '''\
                    # Set up Rust toolchain.
                    rustup default 1.53-x86_64-pc-windows-gnu

                    # Build binary and run it to get version.
                    version="$(cargo run --release -- --version | awk '{print $2}')"

                    # Push binary to S3.
                    aws s3 cp \
                        ./target/release/concordium-rosetta.exe \
                        "s3://distribution.concordium.software/tools/windows/concordium-rosetta_${version}.exe" \
                        --grants=read=uri=http://acs.amazonaws.com/groups/global/AllUsers
                '''.stripIndent()
            }
        }
    }
}
