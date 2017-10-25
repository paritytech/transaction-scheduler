import React, { Component } from 'react'
import { Container, Header } from 'semantic-ui-react'

export default class Docs extends Component {
  render () {
    return (
      <Container text style={{ marginTop: '2rem' }}>
        <Header as='h1'>RPC Documentation</Header>
        <p>Transaction Scheduler exposes a JSON-RPC methods allowing you to integrate your app with it.</p>
      </Container>
    )
  }
}
