use std::cmp::Ordering;

#[repr(C)]
struct BlkIOTrace {
    pub magic: u32,
    pub sequence: u32,
    pub time: u64,
    pub sector: u64,
    pub bytes: u32,
    pub action: u32,
    pub pid: u32,
    pub device: u32,
    pub cpu: u32,
    pub error: u16,
    pub pdu_len: u16,
}

pub const SECTOR_SIZE: usize = 512;

#[derive(FromPrimitive, PartialEq, Eq)]
pub enum Action {
    Other = 0,
    Queue = 1,
    Backmerge = 2,
    Frontmerge = 3,
    GetRQ = 4,
    SleepRQ = 5,
    Requeue = 6,
    Issue = 7,
    Complete = 8,
    Plug = 9,
    UnplugIO = 10,
    UnplugTimer = 11,
    Insert = 12,
    Split = 13,
    Bounce = 14,
    Remap = 15,
    Abort = 16,
    DrvData = 17,
}

bitflags! {
    pub struct Category: u16 {
        const READ	= 1 << 0;	/* reads */
        const WRITE	= 1 << 1;	/* writes */
        const FLUSH	= 1 << 2;	/* flush */
        const SYNC	= 1 << 3;	/* sync IO */
        const QUEUE	= 1 << 4;	/* queueing/merging */
        const REQUEUE	= 1 << 5;	/* requeueing */
        const ISSUE	= 1 << 6;	/* issue */
        const COMPLETE	= 1 << 7;	/* completions */
        const FS	= 1 << 8;	/* fs requests */
        const PC	= 1 << 9;	/* pc requests */
        const NOTIFY	= 1 << 10;	/* special message */
        const AHEAD	= 1 << 11;	/* readahead */
        const META	= 1 << 12;	/* metadata */
        const DISCARD	= 1 << 13;	/* discard requests */
        const DRV_DATA	= 1 << 14;	/* binary per-driver data */
        const FUA	= 1 << 15;	/* fua requests */

        const END	= 1 << 15;	/* we've run out of bits! */
    }
}

#[derive(PartialEq, Eq)]
pub struct EventPDU {
    pub data: Vec<u8>,
}

#[derive(PartialEq, Eq)]
pub struct Event {
    pub sequence: u32,
    pub time: u64,
    pub sector: u64,
    pub bytes: u32,
    pub action: Action,
    pub category: Category,
    pub pid: u32,
    pub device: u32,
    pub cpu: u32,
    pub error: u16,
    pub pdu: Option<EventPDU>,
}

impl Event {
    fn from_raw(trace: &BlkIOTrace, pdu_data: &[u8]) -> Event {
        use super::num;
        let pdu = {
            if pdu_data.len() > 0 {
                Some(EventPDU {
                    data: {
                        let mut t = Vec::new();
                        t.extend_from_slice(pdu_data);
                        t
                    },
                })
            } else {
                None
            }
        };
        Event {
            sequence: trace.sequence,
            time: trace.time,
            sector: trace.sector,
            bytes: trace.bytes,
            action: num::FromPrimitive::from_u32(trace.action & 0xffff)
                .expect("invalid action type"),
            category: Category::from_bits_truncate(((trace.action & 0xffff0000) >> 16) as u16),
            pid: trace.pid,
            device: trace.device,
            cpu: trace.cpu,
            error: trace.error,
            pdu: pdu,
        }
    }

    pub fn ending_sector(&self) -> u64 {
        self.sector + (self.bytes as f64 / SECTOR_SIZE as f64).ceil() as u64
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Trace {
    pub events: Vec<Event>,
}

unsafe fn as_trace<'a>(b: &'a [u8]) -> &'a BlkIOTrace {
    use std::mem;
    assert!(b.len() == mem::size_of::<BlkIOTrace>());
    let s: *const BlkIOTrace = b.as_ptr() as *const _;
    s.as_ref().unwrap()
}

fn parse(b: &[u8]) -> Vec<Event> {
    use std::mem;
    const STEP_SIZE: usize = mem::size_of::<BlkIOTrace>();
    let mut index: usize = 0;
    let mut events = Vec::new();
    while index + STEP_SIZE < b.len() {
        let trace = unsafe { as_trace(&b[index..index + STEP_SIZE]) };
        index += STEP_SIZE;
        let pdu_data = &b[index..index + trace.pdu_len as usize];
        let event = Event::from_raw(trace, pdu_data);
        events.push(event);
        // NOTE: we do not check if the trace is valid
        index += trace.pdu_len as usize;
    }
    events
}

impl Trace {
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        let mut events = data.iter().fold(Vec::new(), |mut acc, s| {
            acc.append(&mut parse(s));
            acc
        });
        events.sort();
        Self { events: events }
    }
}
