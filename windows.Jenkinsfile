pipeline {
    agent { label 'windows' }

    stages {
        stage('build') {
            steps {
                sh '''\
                    # Set up Rust toolchain.
                    rustup default 1.53-x86_64-pc-windows-gnu

                    # Build and run binary to get version.
                    ls
                    version=$(cargo run --release -- --version | awk '{print $2}')
                    ls
                    ls target
                    ls target/release

                    # Push binary to S3.
                    aws s3 cp \
                        ./target/release/concordium-rosetta.exe \
                        s3://distribution.concordium.software/tools/windows/ \
                        --grants=read=uri=http://acs.amazonaws.com/groups/global/AllUsers
                '''.stripIndent()
            }
        }
    }
}
