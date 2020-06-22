import 'regenerator-runtime/runtime';
import React, { Component } from 'react';
import { Button ,Card ,Spin,List ,Modal ,Layout ,Input ,message,notification ,Carousel } from 'antd';
const { Header ,Sider, Content} = Layout;
const { Meta } = Card;
import './App.css';

class App extends Component {
  constructor(props) {
    super(props);
    this.state = {
      amount:'0.00',
      visible: false,
      visible_tranfer: false,
      reciver_transfer:"",
      transfer_token_id:"",
      token_id_arr:[],
      img_arr: [
        {"title":"first","token_id":"a001","src":"http://qiniu.eth.fm/2020-03-18-WechatIMG60.jpeg"},
        {"title":"first","token_id":"a001","src":"http://mathwallet.oss-cn-hangzhou.aliyuncs.com/blog/2020/3%E6%9C%88/NFT/NTF-Gif-800.gif"},
        {"title":"first","token_id":"a001","src":"http://mathwallet.oss-cn-hangzhou.aliyuncs.com/blog/2020/3%E6%9C%88/NFT/%23003.jpg"},
        {"title":"first","token_id":"a001","src":"http://mathwallet.oss-cn-hangzhou.aliyuncs.com/blog/2020/4%E6%9C%88/%23004.jpg"},
        {"title":"first","token_id":"a001","src":"http://mathwallet.oss-cn-hangzhou.aliyuncs.com/blog/2020/4%E6%9C%88/%23005.jpg"},
        {"title":"first","token_id":"a001","src":"http://mathwallet.oss-cn-hangzhou.aliyuncs.com/blog/2020/NFT_IMG/NFT_stakefish.gif"},
        {"title":"first","token_id":"a001","src":"https://cdn.enjinx.io/metadata/raw/c986a119f987a0ab06665484372e8b365f13d3fb/bd54065b18e0e29d2c8e49dacc847fb1aa3a7f18.jpeg?width=250px&height=250px"},
        {"title":"first","token_id":"a001","src":"https://cdn.enjinx.io/metadata/raw/dec3ede0e86cf62fbbe689d0fcaa0a9b5146b1a5/28d71ef74250eef6339a1618cc90d6b809fca21b.jpeg?width=250px&height=250px"},
      ],
      stamp_arr:[],
      login: false,
      speech: null,
      token_info:"",
      token_owner:"",
    }
    this.signedInFlow = this.signedInFlow.bind(this);
    this.requestSignIn = this.requestSignIn.bind(this);
    this.requestSignOut = this.requestSignOut.bind(this);
    this.signedOutFlow = this.signedOutFlow.bind(this);
  }

  componentDidMount = async ()=> {
    await this.getTotal();
    this.getAccount(this.props.accountId);
    this.initStampData();
    let loggedIn = this.props.wallet.isSignedIn();
    if (loggedIn) {
      this.signedInFlow();
    } else {
      this.signedOutFlow();
    }
  }
  getTotal = async ()=>{
    let total = await this.props.contract.total();
    let arrs = [];
    for(var i=1;i<= total;i++){
      let token_id = await this.props.contract.count_id_token({"id":i});
      arrs.push(token_id)
    }
    this.setState({
      token_id_arr: arrs
    })
  }
  getAccount = async (accountId)=>{
    let account = await this.checkAccount(accountId)
    if(account){
      let account_detail = await account.state();
      this.state.amount = account_detail.amount;
      this.setState({
        amount: Number(account_detail.amount/10**24).toFixed(4)
      })
      console.log("account_detail--",account_detail)
    }
    
  }
  async checkAccount(accounts){
    try {
      let acc = await this.props.near.account(accounts);
      return acc;
    } catch (error) {
    }
    return null;
  }
  initStampData= async ()=>{
    let datas = [];
    for(var i=0;i< this.state.token_id_arr.length;i++){
      let stamp = await this.props.contract.stamp_info({"token_id":this.state.token_id_arr[i]});
      datas.push(stamp);
    }
    
    this.setState({
      stamp_arr: datas
    })
  }
  async signedInFlow() {
    this.setState({
      login: true,
    })
    const accountId = await this.props.wallet.getAccountId()
    if (window.location.search.includes("account_id")) {
      window.location.replace(window.location.origin + window.location.pathname)
    }
  }

  async requestSignIn() {
    const appTitle = 'CBaseChain Stamp project';
    await this.props.wallet.requestSignIn(
      window.nearConfig.contractName,
      appTitle
    )
  }

  requestSignOut() {
    this.props.wallet.signOut();
    setTimeout(this.signedOutFlow, 500);
    console.log("after sign out", this.props.wallet.isSignedIn())
  }
  signedOutFlow() {
    if (window.location.search.includes("account_id")) {
      window.location.replace(window.location.origin + window.location.pathname)
    }
    this.setState({
      login: false,
      speech: null
    })
  }
  showStampInfo = async (tokenId)=>{
    
    let token_owner = await this.props.contract.get_token_owner({"token_id": tokenId});
    this.setState({
      token_owner:token_owner,
      visible:true
    })
  }
  
  handleOk = e => {
    console.log(e);
    this.setState({
      visible: false,
    });
  };

  handleCancel = e => {
    console.log(e);
    this.setState({
      visible: false,
    });
  };

  showTransferInfo = async (tokenId) =>{
    console.log("ddd-ss-"+tokenId)
    if(!this.state.login){
      notification.open({
        message: 'Login Tips',
        duration: 1,
        top:30,
        description:
          'Please Login CBase Online Wallet',
      });
      return;
    }
    this.setState({
      visible_tranfer:true,
      transfer_token_id: tokenId
    })
  }
  handleOk_T = async () => {
    let acc = await this.checkAccount(this.state.reciver_transfer);
    console.log("acc--",acc)
    if(acc){
      this.setState({
        visible_tranfer: false,
      });
      try {
        this.props.contract.transfer({"new_owner_id": this.state.reciver_transfer,"token_id":this.state.transfer_token_id}).then((e,r)=>{
          console.log("e--",e);
          console.log("r--",r);
        }) 
        message
        .loading('Action in progress..', 2)
        .then(() => message.success('Finished', 2))
      } catch (error) {
        console.log("transfer error--",error)
      }
    }
    
  };

  handleCancel_T = e => {
    console.log(e);
    this.setState({
      visible_tranfer: false,
    });
  };
  onChangeReciver = (e)=>{
    this.setState({
      reciver_transfer : e.target.value
    })
  }




  render() {
    let style = {
      fontSize: "1.5rem",
      color: "#0072CE",
      textShadow: "1px 1px #D1CCBD"
    }
    return (
      <div>
        <div><Spin size="large" /></div>
        
        <Layout>
          <Content className="HeaderAccountclass">
          {this.state.login ? 
                <div className="balance_class">
                   AccountId: <span>{this.props.accountId}</span>&nbsp;&nbsp;&nbsp;Balance: <span>{this.state.amount}</span>
                </div>
                : <div></div>}
            </Content>
          <Sider className="loginClass">
            <div>
              {this.state.login ? 
                <div>
                  <Button onClick={this.requestSignOut}>Log out</Button>
                </div>
                : <div><Button onClick={this.requestSignIn}>Log in with NEAR</Button></div>}
            </div>
           </Sider>
        </Layout>
        <div className="HeaderTitleClass">Stamp</div>
        <Carousel className="lunbopic" autoplay>
            <div>
                <h3><img src="https://ss0.bdstatic.com/70cFvHSh_Q1YnxGkpoWK1HF6hhy/it/u=1646461313,652382455&fm=26&gp=0.jpg"  alt="logo" /></h3>
            </div>
            <div>
                <h3><img src="https://ss2.bdstatic.com/70cFvnSh_Q1YnxGkpoWK1HF6hhy/it/u=2643298458,4192916526&fm=15&gp=0.jpg"  alt="logo" /></h3>
            </div>
            <div>
                <h3><img src="https://ss0.bdstatic.com/70cFuHSh_Q1YnxGkpoWK1HF6hhy/it/u=4243158649,499767841&fm=15&gp=0.jpg"  alt="logo" /></h3>
            </div>
        </Carousel>
        <div>
          <List
            grid={{ gutter: 16,
              xs: 1,
              sm: 2,
              md: 4,
              lg: 4,
              xl: 4,
              xxl: 3,}}
            dataSource={this.state.stamp_arr}
            renderItem={(item,index) => (
              <List.Item>
                <Card
                  key={index}
                  cover={<img alt="example" src={item.image_src} />}
                  actions={[
                    <Button onClick={()=>this.showStampInfo(item.token_id)}>CheckOwner</Button>,
                    <Button type="primary" onClick={()=>this.showTransferInfo(item.token_id)}>Transfer</Button>
                  ]}
                >
                  <p>ID: {item.token_id}</p>
                  <p>Price: {item.price}</p>
                  <Meta title={item.stamp_desc}/>
                </Card>
              </List.Item>
            )}
          />
        </div>
        <Modal
          title="Token Onwer"
          visible={this.state.visible}
          onOk={this.handleOk}
          onCancel={this.handleCancel}
          footer={[]}
        >
          <p>{this.state.token_owner}</p>
        </Modal>
        <Modal
          title="Token Onwer"
          visible={this.state.visible_tranfer}
          onCancel={this.handleCancel_T}
          footer={[(<Button key="cancel_t" onClick={this.handleCancel_T}>Cancel</Button>),(<Button key="confirm_t" onClick={this.handleOk_T} type="primary">Confirm</Button>)]}
        >
          Reciver: <Input onChange={(e)=>this.onChangeReciver(e)} placeholder="please input receiver"/>
        </Modal>
        
      </div>
    )
  }

}

export default App;
