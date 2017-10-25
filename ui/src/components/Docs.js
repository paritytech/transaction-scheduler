import React, { Component } from 'react'
import { Container, Header } from 'semantic-ui-react'

import Scheduler from './Scheduler'

export default class Docs extends Component {
  static defaultProps = {
    rlp: '0x<signed-transaction-rlp>'
  }

  state = {
    condition: { block: 4000000 }
  }

  handleConditions = condition => this.setState({ condition })

  request () {
    return {
      jsonrpc: '2.0',
      id: 1,
      method: 'scheduleTransaction',
      params: [
        this.state.condition,
        this.props.rlp
      ]
    }
  }

  render () {
    const { condition } = this.state
    const request = this.request()
    const domain = `${window.location.protocol}://${window.location.host}`

    return (
      <Container text style={{ marginTop: '2rem' }}>
        <Header as='h1'><a name="docs">RPC Documentation</a></Header>
        <p>Transaction Scheduler exposes a JSON-RPC methods allowing you to integrate your app with it.</p>
        <Scheduler onNewConditions={ this.handleConditions } />

        <Header as='h3'>RPC Request</Header>
        <pre className='code'>{ JSON.stringify(request, null, 2) }</pre>

        <Header as='h3'>CURL example</Header>
        <div className='code'>$ curl {domain} -X POST -HContent-Type:application/json --data '{ JSON.stringify(request) }'</div>
      </Container>
    )
  }
}
