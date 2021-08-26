import "./App.css";
import React from 'react';
// import BN from 'bn.js';
import * as nearAPI from 'near-api-js'

// const OneNear = new BN("1000000000000000000000000");
// const ContractName = 'dev-1629947058100-39317342737943';
const ContractName = 'dev-1629962930296-20948195726082';

class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      connected: false,
      signedIn: false,
      accountId: null,
      digitals: [],
      all_digitals: [],
      own_digital: "",
      target_digital: "",
    };

    this._digitalRefreshTimer = null;

    this.handleOwnChange = this.handleOwnChange.bind(this);
    this.handleTargetChange = this.handleTargetChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);

    this._initNear().then(() => {
      this.setState({
        connected: true,
        signedIn: !!this._accountId,
        accountId: this._accountId,
      });
    });
  }

  componentDidMount() {
  }


  async query_digitals() {
    let digitals = await this._contract.get_digitals({ accountId: this._accountId })
    let all_digitals = await this._contract.get_all_digitals({ accountId: this._accountId })
    this.setState({
      digitals: digitals,
      all_digitals: all_digitals,
    })
  }

  async refreshAccountStats() {
    let digitals = await this._contract.get_digitals({ accountId: this._accountId });
    let all_digitals = await this._contract.get_all_digitals({ accountId: this._accountId })
    if (this._digitalRefreshTimer) {
      clearInterval(this._digitalRefreshTimer);
      this._digitalRefreshTimer = null;
    }

    this.setState({
      digitals: digitals,
      all_digitals: all_digitals,
    });

    // this._digitalRefreshTimer = setInterval(() => {
    //   this.query_digitals();
    // }, 100);
  }



  async _initNear() {
    const nearConfig = {
      networkId: 'default',
      nodeUrl: 'https://rpc.testnet.near.org',
      contractName: ContractName,
      walletUrl: 'https://wallet.testnet.near.org',
    };
    const keyStore = new nearAPI.keyStores.BrowserLocalStorageKeyStore();
    const near = await nearAPI.connect(Object.assign({ deps: { keyStore } }, nearConfig));
    this._keyStore = keyStore;
    this._nearConfig = nearConfig;
    this._near = near;

    this._walletConnection = new nearAPI.WalletConnection(near, ContractName);
    this._accountId = this._walletConnection.getAccountId();

    this._account = this._walletConnection.account();
    this._contract = new nearAPI.Contract(this._account, ContractName, {
      viewMethods: ['get_digitals', 'next_digital', 'get_all_digitals'],
      changeMethods: ['add_first', 'pk'],
    });
    if (this._accountId) {
      await this.refreshAccountStats();
    }
  }


  async requestSignIn() {
    const appTitle = 'Digital War';
    await this._walletConnection.requestSignIn(
      ContractName,
      appTitle
    )
  }

  async logOut() {
    this._walletConnection.signOut();
    this._accountId = null;
    this.setState({
      signedIn: !!this._accountId,
      accountId: this._accountId,
    })
  }

  async addFirst() {
    let res = await this._contract.add_first();
    alert(res);
    window.location.reload()
  }

  async pk_action(){
    let res = await this._contract.pk({own_digital: parseInt(this.state.own_digital, 10), target_digital: parseInt(this.state.target_digital, 10)})
    // alert('own_digital: ' + this.state.own_digital  + ',target_digital: ' + this.state.target_digital);
    alert(res)
    window.location.reload()
  }

  handleOwnChange(event) {    this.setState({own_digital: event.target.value});  }
  handleTargetChange(event) {    this.setState({target_digital: event.target.value});  }
  handleSubmit(event) {
    this.pk_action();
    event.preventDefault();
  }

  render() {
    const content = !this.state.connected ? (
      <div>Connecting... <span className="spinner-grow spinner-grow-sm" role="status" aria-hidden="true"></span></div>
    ) : (this.state.signedIn ? (
      <div>
        <div className="float-right">
          <button
            className="btn btn-outline-secondary"
            onClick={() => this.logOut()}>Log out</button>
        </div>
        <h4>Hello, <span className="font-weight-bold">{this.state.accountId}</span>!</h4>

        <form onSubmit={this.handleSubmit}>
          <label> challenger: <input type="text" style={{marginLeft:"44px"}} value={this.state.own_digital} onChange={(event) => {this.handleOwnChange(event)}} /> </label><br/>
          <label> challenge target: <input type="text" value={this.state.target_digital} onChange={(event) => {this.handleTargetChange(event)}} /> </label><br/>
          <input type="submit" value="challenge" />
        </form>
        <hr></hr>
        <div>
          Your digitals:
          <ul>
            {this.state.digitals.length !== 0 ?
              (this.state.digitals.map(value => <li key={value}> {value}</li>)) :
              (<button className="btn btn-primary" onClick={() => this.addFirst()}>Get fist digital</button>)}
          </ul>
        </div>
        <hr></hr>
        <div>
          Digital Pool:
          <ul>
            {this.state.all_digitals.length !== 0 ?
              (this.state.all_digitals.map(value => <li style={{whiteSpace: "pre"}} key={value.digital}> Digital：{value.digital}       Owner：{value.owner}       Level：{value.level}</li>)) :
              (<div>暂无</div>)}
          </ul>
        </div>
      </div>
    ) : (
      <div style={{ marginBottom: "10px" }}>
        <button
          className="btn btn-primary"
          onClick={() => this.requestSignIn()}>Log in with NEAR Wallet</button>
      </div>
    ));
    return (
      <div className="px-5">
        <h1>Digital War</h1>
        {content}
      </div>
    );
  }
}

export default App;
