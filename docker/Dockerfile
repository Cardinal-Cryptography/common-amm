FROM public.ecr.aws/p6e8q1z1/aleph-node:r-11.4

# 1. Save node and sudo account ids for convenience
# 2. Prepare the chainspec
# 3. Save bootnode peer id for convenience
RUN aleph-node key inspect //Alice | grep "SS58 Address" | awk "{print \$3}" \
  > /node/sudo_account_id \
  && \
    aleph-node key inspect //1 | grep "SS58 Address" | awk "{print \$3}" \
      > /node/node_account_id \
  && \
    aleph-node bootstrap-chain --base-path /node \
      --account-ids $(cat /node/node_account_id) \
      --min-validator-count 1 > chainspec.json \
  && \
    aleph-node key inspect-node-key \
      --file /node/$(cat /node/node_account_id)/p2p_secret \
      > /node/bootnode_peer_id

ENTRYPOINT aleph-node -lerror,runtime::contracts=debug \
  --chain chainspec.json \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/$(cat /node/bootnode_peer_id) \
  --base-path /node/$(cat /node/node_account_id)/ \
  --node-key-file /node/$(cat /node/node_account_id)/p2p_secret \
  --validator \
  --execution Native \
  --public-validator-addresses 192.0.0.1:5678 \
  --ws-port 9944
