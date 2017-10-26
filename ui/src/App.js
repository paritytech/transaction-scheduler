import React, { Component } from 'react'
import {
  Container, Header, Visibility, Menu, Image, Divider
} from 'semantic-ui-react'

import Compose from './components/Compose'
import Docs from './components/Docs'
import SendRaw from './components/SendRaw'

import logo from './logo.svg'
import './App.css'

class App extends Component {
  state = {
    menuFixed: false,
    rlp: '0x<signed-transaction-rlp>',
    condition: { block: '0x12345' },
  }

  stickTopMenu = () => this.setState({ menuFixed: true })
  unStickTopMenu = () => this.setState({ menuFixed: false })

  handleRlpChange = rlp => this.setState({ rlp })
  handleCondition = condition => this.setState({ condition })

  render () {
    const { rlp, condition } = this.state

    return (
      <div>
        <Container text style={{ marginTop: '2rem' }}>
          <Header as='h1'>Parity Transaction Scheduler</Header>
          <p>Parity Transaction Scheduler allows you to submit a pre-signed transaction to be released to the network (propagated to peers) at specific time or block.</p>
          <ul>
            <li><a href="https://txsched.parity.io">Foundation Scheduler</a></li>
            <li><a href="https://txsched-kovan.parity.io">Kovan Scheduler</a></li>
          </ul>
        </Container>

        { this.renderMenu() }

        { <Compose
          onRlpChange={this.handleRlpChange}
        /> }
        { <SendRaw
          condition={condition}
          onNewCondition={this.handleCondition}
          onRlpChange={this.handleRlpChange}
          rlp={rlp}
        /> }
        <Divider />
        { <Docs
          condition={condition}
          onNewCondition={this.handleCondition}
          rlp={rlp}
        /> }
      </div>
    )
  }

  renderMenu () {
    const { menuFixed } = this.state

    return (
      <Visibility
        onBottomPassed={this.stickTopMenu}
        onBottomVisible={this.unStickTopMenu}
        once={false}
      >
        <Menu
          borderless
          fixed={menuFixed ? 'top' : null}
          style={menuFixed ? styles.fixedMenu : styles.menu}
          >
          <Container text>
            <Menu.Item>
              <Image size='mini' src={logo} />
            </Menu.Item>
            <Menu.Item as='a' href="#compose">Create Transaction</Menu.Item>
            <Menu.Item as='a' href="#raw">Schedule RAW transaction</Menu.Item>
            <Menu.Menu position='right'>
              <Menu.Item as='a' href="#docs">RPC Docs</Menu.Item>
            </Menu.Menu>
          </Container>
        </Menu>
      </Visibility>
    )
  }
}

const styles = {
  menu: {
    background: 'none',
    border: 'none',
    boxShadow: 'none',
    marginBottom: '2rem',
    marginTop: '2rem',
    transition: 'box-shadow 0.3s ease, padding 0.3s ease'
  },
  menuFixed: {
    backgroundColor: '#fff',
    border: '1px solid #ddd',
    boxShadow: '0px 3px 5px rgba(0,0,0,0.2)'
  }
}

export default App
