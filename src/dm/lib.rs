#![feature(type_ascription)]

extern crate rand;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use context::DMContext;
use global_state::GLOBAL_STATE;
use value::EitherValue;
use value::Value;

mod byond_ffi;
mod context;
mod global_state;
mod hooks;
mod proc;
mod raw_types;
mod string;
mod value;

fn random_string(n: usize) -> String {
	thread_rng().sample_iter(&Alphanumeric).take(n).collect()
}

byond_ffi_fn! { auxtools_init(_input) {
	// Already initialized. Just succeed?
	if GLOBAL_STATE.get().is_some() {
		return Some("SUCCESS".to_owned());
	}

	let byondcore = match sigscan::Scanner::for_module("byondcore.dll") {
		Some(v) => v,
		None => return Some("FAILED (Couldn't create scanner for byondcore.dll)".to_owned())
	};

	let string_table: *mut raw_types::strings::StringTable;
	if let Some(ptr) = byondcore.find(b"\xA1????\x8B\x04?\x85\xC0\x0F\x84????\x80\x3D????\x00\x8B\x18") {
		unsafe {
			// TODO: Could be nulls
			string_table = *(ptr.offset(1) as *mut *mut raw_types::strings::StringTable);
		}
	} else {
		return Some("FAILED (Couldn't find stringtable)".to_owned())
	}

	let get_proc_array_entry: raw_types::funcs::GetProcArrayEntry;
	if let Some(ptr) = byondcore.find(b"\xE8????\x8B\xC8\x8D\x45?\x6A\x01\x50\xFF\x76?\x8A\x46?\xFF\x76?\xFE\xC0") {
		unsafe {
			// TODO: Could be nulls
			let offset = *(ptr.offset(1) as *const isize);
			get_proc_array_entry = std::mem::transmute(ptr.offset(5).offset(offset) as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetProcArrayEntry)".to_owned())
	}

	let get_string_id: raw_types::funcs::GetStringId;
		if let Some(ptr) = byondcore.find(b"\x55\x8B\xEC\x8B\x45?\x83\xEC?\x53\x56\x8B\x35") {
		unsafe {
			// TODO: Could be nulls
			get_string_id = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetStringId)".to_owned())
	}

	let call_proc_by_id: raw_types::funcs::CallProcById;
	if let Some(ptr) = byondcore.find(b"\x55\x8B\xEC\x81\xEC????\xA1????\x33\xC5\x89\x45?\x8B\x55?\x8B\x45") {
		unsafe {
			// TODO: Could be nulls
			call_proc_by_id = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find CallGlobalProc)".to_owned())
	}

	let get_variable: raw_types::funcs::GetVariable;
	if let Some(ptr) = byondcore.find(b"\x55\x8B\xEC\x8B\x4D?\x0F\xB6\xC1\x48\x83\xF8?\x0F\x87????\x0F\xB6\x80????\xFF\x24\x85????\xFF\x75?\xFF\x75?\xE8") {
		unsafe {
			// TODO: Could be nulls
			get_variable = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetVariable)".to_owned())
	}

	let set_variable: raw_types::funcs::SetVariable;
	if let Some(ptr) = byondcore.find(b"\x55\x8B\xEC\x8B\x4D\x08\x0F\xB6\xC1\x48\x57\x8B\x7D\x10\x83\xF8\x53\x0F?????\x0F\xB6\x80????\xFF\x24\x85????\xFF\x75\x18\xFF\x75\x14\x57\xFF\x75\x0C\xE8????\x83\xC4\x10\x5F\x5D\xC3") {
		unsafe {
			// TODO: Could be nulls
			set_variable = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find SetVariable)".to_owned())
	}

	let get_string_table_entry: raw_types::funcs::GetStringTableEntry;
	if let Some(ptr) = byondcore.find(b"\x55\x8B\xEC\x8B\x4D\x08\x3B\x0D????\x73\x10\xA1") {
		unsafe {
			// TODO: Could be nulls
			get_string_table_entry = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find GetStringTableEntry)".to_owned())
	}

	let call_datum_proc_by_name: raw_types::funcs::CallDatumProcByName;
	if let Some(ptr) = byondcore.find(b"\x55\x8B\xEC\x83\xEC\x0C\x53\x8B\x5D\x10\x8D\x45\xFF\x56\x8B\x75\x14\x57\x6A\x01\x50\xFF\x75\x1C\xC6\x45\xFF\x00\xFF\x75\x18\x6A\x00\x56\x53") {
		unsafe {
			// TODO: Could be nulls
			call_datum_proc_by_name = std::mem::transmute(ptr as *const ());
		}
	} else {
		return Some("FAILED (Couldn't find CallDatumProcByName)".to_owned())
	}

	/*
	char* x_ref_count_call = (char*)Pocket::Sigscan::FindPattern(BYONDCORE, "3D ?? ?? ?? ?? 74 14 50 E8 ?? ?? ?? ?? FF 75 0C FF 75 08 E8", 20);
	DecRefCount = (DecRefCountPtr)(x_ref_count_call + *(int*)x_ref_count_call + 4); //x_ref_count_call points to the relative offset to DecRefCount from the call site
	x_ref_count_call = (char*)Pocket::Sigscan::FindPattern(BYONDCORE, "FF 75 10 E8 ?? ?? ?? ?? FF 75 0C 8B F8 FF 75 08 E8 ?? ?? ?? ?? 57", 17);
	IncRefCount = (IncRefCountPtr)(x_ref_count_call + *(int*)x_ref_count_call + 4);
	*/
	let dec_ref_count: raw_types::funcs::DecRefCount;
	let inc_ref_count: raw_types::funcs::IncRefCount;
	unsafe {
		let dec_ref_count_call: *const u8 = byondcore.find(b"\x3D????\x74\x14\x50\xE8????\xFF\x75\x0C\xFF\x75\x08\xE8").unwrap().offset(20);
		dec_ref_count = std::mem::transmute(dec_ref_count_call.offset((*(dec_ref_count_call as *const u32) + 4) as isize));

		let inc_ref_count_call: *const u8 = byondcore.find(b"\xFF\x75\x10\xE8????\xFF\x75\x0C\x8B\xF8\xFF\x75\x08\xE8????\x57").unwrap().offset(17);
		inc_ref_count = std::mem::transmute(inc_ref_count_call.offset((*(inc_ref_count_call as *const u32) + 4) as isize));
	}

	if GLOBAL_STATE.set(global_state::State {
		get_proc_array_entry: get_proc_array_entry,
		get_string_id: get_string_id,
		execution_context: std::ptr::null_mut(),
		string_table: string_table,
		call_proc_by_id: call_proc_by_id,
		get_variable: get_variable,
		set_variable: set_variable,
		get_string_table_entry: get_string_table_entry,
		call_datum_proc_by_name: call_datum_proc_by_name,
		dec_ref_count: dec_ref_count,
		inc_ref_count: inc_ref_count

	}).is_err() {
		panic!();
	}

	if let Err(error) = hooks::init() {
		return Some(error);
	}

	proc::populate_procs();

	hooks::hook("/proc/wew", hello_proc_hook).unwrap_or_else(|e| {
			msgbox::create("Failed to hook!", e.to_string().as_str(), msgbox::IconType::Error)
		}
	);

	Some("SUCCESS".to_owned())
} }

macro_rules! args {
    () => {
        None
    };
    ($($x:expr),+ $(,)?) => {
        Some(vec![$(value::EitherValue::from($x),)+])
    };
}

fn hello_proc_hook<'a>(
	ctx: &'a DMContext,
	src: Value<'a>,
	usr: Value<'a>,
	args: &Vec<Value<'a>>,
) -> EitherValue<'a> {
	let dat = args[0];

	let string: string::StringRef = "penis".into();
	let string2: string::StringRef = "penisaaa".into();

	string.into()
}

#[cfg(test)]
mod tests {
	#[test]
	fn test() {}
}
