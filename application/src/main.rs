
struct ValidMessage{
  length: u32,
  content: String
}

enum Message{
	Empty,
	Fail,
	Valid (ValidMessage)
}

const fn is_big_endian() -> bool{
	let raw : usize = 0x255;
	let cmp : usize = 0x255usize.to_be();
	return raw == cmp
}

fn write_padded_string_to_stdout(text :&str) -> std::io::Result<()>{
	use std::io::Write;
	let stdout = std::io::stdout();
	let mut handle = stdout.lock();

	let bytes = text.as_bytes();
	let mut length = bytes.len() + 2;

	let mut buf : Vec<u8> = vec![0; 4 + length];
	
	buf[4] = 34;
	let mut idx = 5;
	let mut double_quote_open = true;
	let mut single_quote_open = false;
	for byte in bytes{
		let mut needs_escape = true;
		let new : &u8 = match byte{
			&92u8 => &92,	// \	-> \
			&9u8 => &114,	// TAB	-> r
			&10u8 => &110,	// LF	-> n 
			&13u8 => &116,	// CR	-> t
			&34u8 => {		// "	-> "
				if single_quote_open{
					needs_escape = false
				}else{
					double_quote_open = !double_quote_open;
				}
				&34
			},
			&39u8 => {	// '	-> '
				if double_quote_open{
					needs_escape = false
				}else{
					single_quote_open = !single_quote_open; 
				}
				&39
			},
			z 	  => { needs_escape = false; z }
		};
		
		if needs_escape{
			buf.push(0);
			buf[idx] = 92;
			idx += 1;
			length += 1;
		}
		buf[idx] = *new;
		
		idx += 1;
	}
	buf[idx] = 34;

	buf[0] = (length & 0xff) as u8;
	buf[1] = ((length >> 8) & 0xff) as u8;
	buf[2] = ((length >> 16) & 0xff) as u8;
	buf[3] = ((length >> 24) & 0xff) as u8;
	if is_big_endian(){
		buf.swap(0,3);
		buf.swap(1,2);
	}

	handle.write_all(&buf)?;
	handle.flush()?;
	
	Ok(())
}

struct Stout{ }
impl Stout{
	fn error (text : &str,kind : std::io::ErrorKind) -> (){
		let st = String::from(text) + kind.ek_str();
		Stout::try_write(st)
	}
	fn info ( text: &str, data: &str ) -> () {
		let st = String::from(text) + data;
		Stout::try_write(st)
	}
	fn try_write (st : String) -> (){
		match write_padded_string_to_stdout(st.as_str()){
			Ok(()) => (),
			Err(_) => ()
		}
	}
}

impl ValidMessage{
	fn to_stdout(&self) -> (){
		match write_padded_string_to_stdout(self.content.as_str()){
			Ok(_) => (),
			Err(e) => Stout::error("Error writing to stdout: ", e.kind())
		}
	}
}

impl Message{

	fn new(text : String) -> Message {
		Message::Valid( ValidMessage{ length: text.len() as u32, content: text } )
	}

	fn to_stdout(&self){
		match self{
			Message::Valid(x) => x.to_stdout(),
			_ => ()
		}
	}

	fn from_stdin() -> Message{
		use std::io::Read;
		// parse first 32bits of buffer
		let stdin = std::io::stdin();
	    let iolock = stdin.lock();
		let mut nth_byte : u32 = 0;
		let mut message_length : u32 = 0;
		let mut limit : u32 = 1000;
		let mut vec = Vec::new();
		
		for read in iolock.bytes(){
			match read{
				Ok(byte) => {
					match nth_byte{
						0 | 1 | 2 | 3 => message_length += (byte as u32) << (nth_byte * 8),
						4 => {
							message_length = u32::from_le(message_length);
							if message_length < limit +4 {
								limit = message_length + 4; // +4 for the first 4 bytes
							}
							if limit > 4{
								vec.push(byte)
							}
						},
						_ => {
							match byte{
								10 | 13 => break,
								u => vec.push(u)
							};
						}
					}
				},
				Err(e) => {
					Stout::error("Couldn't read stdin: ", e.kind());
					return Message::Fail
				}
			}
			nth_byte += 1;
			if nth_byte >= limit{
				break
			}
		}

		if vec.len() > 2 && vec.ends_with(&[34u8]) && vec.starts_with(&[34u8]){
			let end = vec.len() - 1;
			vec.copy_within(1..end,0);
			vec.pop();
			vec.pop();
		}
		
		match String::from_utf8(vec){
			Ok(s) => Message::Valid( ValidMessage{ length : s.len() as u32, content: s }),
			Err(_) => Message::Empty
		}
	} 
}

fn read_config() -> std::io::Result<String> {
	use std::io::Read;
	let mut f = std::fs::File::open("rsio.conf")?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;
	Ok(buffer)
}

struct Config<'a>{
	name : &'a str,
	exec_path : &'a str
}

impl Config<'_>{
	fn new<'a>() -> Config<'a>{
		Config{ name: (&"").to_owned(), exec_path: (&"").to_owned() }
	}
	fn to_string(&self) -> String{
		return String::from(self.name) + "\n" + self.exec_path
	}
}


fn parse_config(content : &String) -> Config{
	let mut conf = Config::new();
	
	for line in content.lines(){
		let parts : Vec<&str> = line.split('=').collect();
		if parts.len() < 2{
			continue
		}
		match parts[0]{
			"name" => conf.name = (&parts[1]).to_owned(),
			"exec_path" => conf.exec_path = (&parts[1]).to_owned(),
			_ => {}
		}
	}
	return conf
}

fn main() {
	let conf = read_config();
	match conf{
		Ok(x) => main_loop(&parse_config(&x)),
		Err(_) => main_loop(&Config::new())
		
	};
	return ()
}

fn write_file(mes : TaskWritable) -> std::io::Result<String> {
	use std::io::Write;
	let mut buffer = std::fs::File::create(&mes.filename)?;
	buffer.write_all(&mes.data)?;
	buffer.flush()?;
	Ok(mes.filename)
} 

trait EkStr{
	fn ek_str(&self) -> &str;
}

impl EkStr for std::io::ErrorKind{
	fn ek_str(&self) -> &'static str{
		match self{
			std::io::ErrorKind::NotFound			=> "Not found",
			std::io::ErrorKind::PermissionDenied	=> "Permission denied",
			std::io::ErrorKind::AlreadyExists		=> "File exists",
			std::io::ErrorKind::InvalidInput		=> "Invalid input",
			std::io::ErrorKind::InvalidData			=> "Invalid data",
			std::io::ErrorKind::TimedOut			=> "Timeout",
			std::io::ErrorKind::UnexpectedEof 		=> "unexpected EOF",
			_ 										=> "Unknown error"
		}
	}
}

fn try_write_file(_config : &Config, writeable : TaskWritable) -> (){
	match write_file(writeable){
		Ok(x) => Stout::info("wrote file: ", x.as_str()),
		Err(e) => Stout::error("Error writing file: ",e.kind())
	}
}

fn try_canonical(some: &str) -> std::io::Result<()>{
	std::fs::canonicalize(some)?;
	Ok(())
}

struct Runnable{
	command : String,
	dir : String
}

impl Runnable{
	fn new( a_command : &str, a_directory : &str ) -> Runnable{
		let base = String::from(a_directory);
		Runnable{ command: base.clone() + a_command, dir: base  }
	}
}

fn compose_command<'a>(path : &'a str, command: &'a str) -> Option<Runnable>{
	match try_canonical(command){
		Ok(()) => Some(Runnable::new(command,"./")),
		Err(_) => {
			let t = String::from(path) + command;
			match try_canonical(&t){
				Ok(()) => Some(Runnable::new(command,path)),
				Err(_) => None
			}
		}
	}
}

fn do_stuff(config : &Config,task : TaskExecutable) -> () {

	match compose_command(&config.exec_path, &task.command){
		Some(command) => {
			let child = std::process::Command::new(&command.command)
			.current_dir(&command.dir)
			.stdout(std::process::Stdio::null())
			.args(&task.args)
			.spawn();
		
			match child{
				Ok(_) => Stout::info("Doing stuff: ", &command.command),
				Err(e) => Stout::error("command failed: ",e.kind())
			};
			return ()
		},
		None => Stout::info("command path couldn't be resolved","")
	}
	return ()
}

enum Taske{
	Empty,
	Ping (ValidMessage),
	Quit,
	Config,
	Mirror (ValidMessage),
	Execute (TaskExecutable),
	Write (TaskWritable),
}

struct TaskWritable{
	filename: String,
	data: Vec<u8>
}

struct TaskExecutable{
	command: String,
	args: Vec<String>
}

impl Taske{
	fn from_message(message : ValidMessage) -> Taske{
		let mut parts = message.content.split(' ');
		let operation = parts.next();
		
		if operation.is_none(){
			return Taske::Empty
		}
		let op_str = operation.unwrap();

		match op_str{
			"quit" => return Taske::Quit,
			"ping" => return Taske::Ping(ValidMessage{content:String::from("pong"),length:4u32}),
			"config" => return Taske::Config,
			"mirror" => return Taske::Mirror(message),
			_ => ()
		}

		let command = parts.next();
		if command.is_none(){
			return Taske::Empty
		}
		match op_str{
			"dostuff" => {
				let mut args : Vec<String> = Vec::new();
				loop {
					let part = parts.next();
					match part{
						Some(x) => args.push(x.to_string()),
						None => break
					}
				}
				return Taske::Execute( TaskExecutable{ command: command.unwrap().to_string(), args: args } );
			}
			"write" => {
				use std::iter::FromIterator;
				let mut bytes : Vec<u8> = Vec::new();
				loop {
					
					let part = parts.next();
					match part{
						Some(word) => bytes.append(&mut Vec::<u8>::from_iter(word.bytes().map(|x| x as u8))),
						None => break
					}
					bytes.push(32);
				}
				return Taske::Write( TaskWritable{ filename: command.unwrap().to_string(), data: bytes } );
			},
			
			_ => return Taske::Empty
		}
	}
}


fn main_loop(config : &Config) -> bool {
	loop {
		std::thread::sleep(std::time::Duration::from_millis(1000));
		
		let task = match Message::from_stdin(){
			Message::Fail => {
				Stout::info("Something happened: ","failure");
				continue
			},
			Message::Empty => {
				Stout::info("Something happened: ","empty");
				continue
			},
			Message::Valid( valid ) => Taske::from_message(valid)
		};

		match task{
			Taske::Empty => (),
			Taske::Quit => return true,
			Taske::Ping( mes ) => ( mes.to_stdout() ),
			Taske::Config => Message::new(config.to_string()).to_stdout(),
			Taske::Mirror( mes ) => ( mes.to_stdout() ),
			Taske::Execute( exec ) => do_stuff(&config,exec),
			Taske::Write( writeable ) => try_write_file(&config,writeable)
		}
	}
}
