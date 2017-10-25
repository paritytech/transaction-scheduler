export function schedule (condition, rlp) {
    return window.fetch('/rpc', {
      method: 'post',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'scheduleTransaction',
        params: [condition, rlp]
      })
    }).then(res => res.json())
}
