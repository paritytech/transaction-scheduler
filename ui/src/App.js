import React, { Component } from 'react'
import {
  Container, Header, Visibility, Menu, Image
} from 'semantic-ui-react'

import Compose from './components/Compose'
import Docs from './components/Docs'
import SendRaw from './components/SendRaw'

import logo from './logo.svg'
import './App.css'

class App extends Component {
  state = {
    menuFixed: false
  }

  stickTopMenu = () => this.setState({ menuFixed: true })
  unStickTopMenu = () => this.setState({ menuFixed: false })

  render () {
    return (
      <div>
        <Container text style={{ marginTop: '2rem' }}>
          <Header as='h1'>Parity Transaction Scheduler</Header>
          <p>Parity Transaction Scheduler allows you to submit a pre-signed transaction to be released to the network (propagated to peers) at specific time or block.</p>
        </Container>

        { this.renderMenu() }

        <div>
          <a name="raw" />
          { <SendRaw /> }
        </div>
        <div>
          <a name="compose" />
          { <Compose /> }
        </div>
        <div>
          <a name="docs" />
          { <Docs /> }
        </div>
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
          fixed={menuFixed && 'top'}
          style={menuFixed ? styles.fixedMenu : styles.menu}
          >
          <Container text>
            <Menu.Item>
              <Image size='mini' src={logo} />
            </Menu.Item>
            {menuFixed ? <Menu.Item header>Transaction Scheduler</Menu.Item> : null }
            <Menu.Item as='a' href="#raw">Send RAW transaction</Menu.Item>
            <Menu.Item as='a' href="#compose">Create Transaction</Menu.Item>

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
    border: 'none',
    boxShadow: 'none',
    marginBottom: '1rem',
    marginTop: '4rem',
    transition: 'box-shadow 0.3s ease, padding 0.3s ease'
  },
  menuFixed: {
    backgroundColor: '#fff',
    border: '1px solid #ddd',
    boxShadow: '0px 3px 5px rgba(0,0,0,0.2)'
  }
}

export default App
