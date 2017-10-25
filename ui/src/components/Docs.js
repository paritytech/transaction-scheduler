import React, { Component } from 'react'
import { Container, Header, Grid } from 'semantic-ui-react'

import Scheduler from './Scheduler'

export default class Docs extends Component {
  request () {
    return {
      jsonrpc: '2.0',
      id: 1,
      method: 'scheduleTransaction',
      params: [
        this.props.condition,
        this.props.rlp
      ]
    }
  }

  render () {
    const { condition, onNewCondition } = this.props
    const request = this.request()
    const domain = `${window.location.protocol}://${window.location.host}`

    return (
      <Container text style={{ marginTop: '2rem' }}>
        <Header as='h1'><a name='docs'>RPC Documentation</a></Header>
        <p>Transaction Scheduler exposes a JSON-RPC methods allowing you to integrate your app with it.</p>
        <Grid doubling columns={2} divided>
          <Grid.Column>
            <Header as='h3'>RPC Request</Header>
            <pre className='code'>{ JSON.stringify(request, null, 2) }</pre>
          </Grid.Column>
          <Grid.Column>
            <Scheduler onNewCondition={onNewCondition} condition={condition} />
          </Grid.Column>
        </Grid>

        <Header as='h3'>CURL example</Header>
        <div className='code'>$ curl {domain} -X POST -HContent-Type:application/json --data '{ JSON.stringify(request) }'</div>
      </Container>
    )
  }
}
