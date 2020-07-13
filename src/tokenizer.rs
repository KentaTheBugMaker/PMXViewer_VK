pub mod Tokenizer{
    use std::io::Read;
    use std::process;

    pub struct Lexar {
    inner: String,
    i:usize
}
#[derive(PartialEq)]
pub struct Token {
    pub kind: String,
    pub value: String,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        format!("kind: {},\t value: {}", self.kind, self.value)
    }
}

impl Lexar {
    //インスタンス生成
    pub fn new(inner:String) -> Self {
        Self {
            inner,
            i:0
        }
    }
    //トークン終了か？
    fn is_eot(&self)->bool{
        self.inner.len()<=self.i
    }
    //指定された位置を読み込む
    fn get_c(&self)->char{
        if self.is_eot(){
            println!("No more character");
            process::exit(1);
        };
        self.inner.chars().nth(self.i).unwrap()
    }
    //次の文字を取ってくる
    fn next(&mut self)->char{
        let c=self.get_c();
        self.i+=1;
        c
    }
    //空白を取り除く
    fn skip_space(&mut self){
        while !self.is_eot()&&self.get_c().is_whitespace(){
            self.next();
        }
    }
    //演算子が始まる
    fn is_sign_start(c: char)->bool{
        match c {
        '+'|
        '-'|
        '*'|
        '/'|
        '='=>true,
            _=>false
        }
    }
    fn is_num_start(c:char)->bool{
        c.is_digit(10)
    }
    fn is_word_start(c:char)->bool{
        c.is_alphabetic()
    }
    fn sign(&mut self)->Token{
        let mut s=String::new();
        s.push(self.next());
        let a=match self.next(){
            '+'|'-'|'*'|'/'|'='=>self.get_c(),
            _=>' '
        };
        s.push(a);
        Token{
            kind: "OP".to_string(),
            value: s.to_string()
        }
    }

    fn num(&mut self)->Token{
        let mut s=String::new();
        s.push(self.next());
        while !self.is_eot()&&(self.get_c().is_digit(10)){
            s.push(self.next());
            let c= self.get_c();
            match c{
                'e'|'.'=>{s.push(c)},
                _=>{}
            };
        }
        Token{
            kind:"num".to_string(),
            value:s.to_string()
        }
    }
    fn word(&mut self)->Token{
        let mut s= String::new();
        s.push(self.next());
        while !self.is_eot() && (self.get_c().is_alphabetic() || self.get_c().is_digit(10)) {
            s.push(self.next());
        }

        Token {
            kind: "word".to_string(),
            value: s.to_string()
        }
    }
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_space();
        if self.is_eot() {
            None
        } else if Self::is_sign_start(self.get_c()) {
            Some(self.sign())
        } else if Self::is_num_start(self.get_c()) {
            Some(self.num())
        } else if Self::is_word_start(self.get_c()) {
            Some(self.word())
        } else {
            println!("Not a character for tokens");
            process::exit(1);
        }
    }

    pub fn tokenize(&mut self) -> Vec<Option<Token>> {
        let mut tokens = Vec::new();
        let mut t = self.next_token();

        while t != None {
            tokens.push(t);
            t = self.next_token();
        }

        tokens
    }
}
}