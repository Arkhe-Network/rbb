import { NextResponse } from 'next/server';

export async function POST(req: Request) {
  try {
    const { prompt } = await req.json();
    // Proxy for actual Rust backend if needed
    // const backendRes = await fetch("http://localhost:9898/api/v1/generate", ...);

    // Simula chamada ao AGI Core
    return NextResponse.json({
      response: `Processed: ${prompt} (simulated)`,
    });
  } catch (error) {
    return NextResponse.json({ error: "Failed" }, { status: 500 });
  }
}
