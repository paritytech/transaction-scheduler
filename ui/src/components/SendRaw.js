import React, { Component } from 'react'
import { Container, Header } from 'semantic-ui-react'

export default class SendRaw extends Component {
  render () {
    return (
      <Container text style={{ marginTop: '2rem' }}>
        <Header as='h1'>Send RAW Transaction</Header>
        <p>If you already have a pre-signed raw transaction you can paste it here and schedule for execution.</p>
      </Container>
    )
  }
}
