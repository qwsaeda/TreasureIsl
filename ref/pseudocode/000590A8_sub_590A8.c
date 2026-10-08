/* ea:   0x000590A8 */
/* name: sub_590A8 */
/* size: 1380 */
int __fastcall sub_590A8(HWAVEOUT *a1, int a2, int a3, char a4, unsigned int a5, unsigned int a6)
{
  int v7; // r7
  HWAVEOUT v9; // r0
  int v10; // r4
  bool v11; // cc
  HWAVEOUT v12; // r0
  HWAVEOUT v13; // r6
  char *v14; // r5
  HWAVEOUT v15; // r0
  unsigned int v16; // r4
  int v17; // r3
  int v18; // r4
  bool v19; // cc
  HWAVEOUT v20; // r0
  HWAVEOUT v21; // r6
  char *v22; // r5
  HWAVEOUT v23; // r0
  unsigned int v24; // r4
  int v25; // r2
  HWAVEOUT v26; // r1
  HWAVEOUT v27; // r0
  int v28; // r4
  unsigned int v29; // r1
  unsigned int v30; // r0
  HWAVEOUT v31; // r0
  unsigned int v32; // r1
  HWAVEOUT v33; // r0
  HWAVEOUT Thread; // r0
  HWAVEOUT EventW; // r0
  HWAVEOUT v36; // r4
  UINT v37; // r6
  HWAVEOUT v38; // r2
  HWAVEOUT v39; // r1
  unsigned __int16 v40; // r3
  int v41; // r2
  MMRESULT v42; // r5
  unsigned int v43; // r5
  int v44; // r4
  _BYTE *v45; // r0
  MMRESULT v46; // [sp+8h] [bp-28h]

  v7 = a2;
  if ( *((_BYTE *)a1 + 68) != 0 )
    return 0;
  switch ( a2 )
  {
    case 11025:
      v9 = (HWAVEOUT)&unk_76A18;
      break;
    case 22050:
      v9 = (HWAVEOUT)&unk_784E0;
      break;
    case 44100:
      v9 = (HWAVEOUT)&unk_79FA8;
      break;
    default:
      v9 = (HWAVEOUT)&unk_784E0;
      v7 = 22050;
      break;
  }
  a1[23] = v9;
  v10 = a5;
  a1[29] = (HWAVEOUT)(a4 != 0);
  v11 = a5 > 1;
  a1[30] = (HWAVEOUT)(a3 == 16);
  if ( a5 != 0 )
    v11 = a5 > 3;
  else
    v10 = 1;
  if ( v11 )
    v10 = 3;
  a1[19] = (HWAVEOUT)v10;
  v12 = (HWAVEOUT)operator new(1304 * v10);
  v13 = v12;
  if ( v12 != nullptr )
  {
    v14 = (char *)v12;
    if ( v10 - 1 >= 0 )
    {
      do
      {
        sub_5AF94(v14);
        --v10;
        v14 += 1304;
      }
      while ( v10 != 0 );
    }
  }
  else
  {
    v13 = nullptr;
  }
  a1[20] = v13;
  if ( v13 == nullptr )
    return 0;
  v15 = a1[19];
  a1[31] = (HWAVEOUT)64;
  v16 = 0;
  if ( v15 != nullptr )
  {
    v17 = 0;
    do
    {
      ++v16;
      *(_DWORD *)((char *)a1[20] + v17 + 4) = a1 + 31;
      v17 += 1304;
    }
    while ( v16 < (unsigned int)a1[19] );
  }
  v18 = a6;
  v19 = a6 > 8;
  if ( a6 >= 8 )
    v19 = a6 > 0x40;
  else
    v18 = 8;
  if ( v19 )
    v18 = 64;
  a1[21] = (HWAVEOUT)v18;
  v20 = (HWAVEOUT)operator new(60 * v18);
  v21 = v20;
  if ( v20 != nullptr )
  {
    v22 = (char *)v20;
    if ( v18 - 1 >= 0 )
    {
      do
      {
        sub_5B084(v22);
        --v18;
        v22 += 60;
      }
      while ( v18 != 0 );
    }
  }
  else
  {
    v21 = nullptr;
  }
  a1[22] = v21;
  if ( v21 == nullptr )
    return 0;
  v23 = a1[21];
  v24 = 0;
  a1[32] = (HWAVEOUT)64;
  if ( v23 != nullptr )
  {
    v25 = 0;
    do
    {
      ++v24;
      *(_DWORD *)((char *)a1[22] + v25 + 4) = a1 + 32;
      v25 += 60;
    }
    while ( v24 < (unsigned int)a1[21] );
  }
  v26 = a1[30];
  a1[7] = (HWAVEOUT)6;
  v27 = a1[29];
  a1[18] = (HWAVEOUT)v7;
  v28 = 1 << (char)v27 << (char)v26;
  a1[27] = (HWAVEOUT)v28;
  v29 = 135 * v28 * v7 / 0x3E8u / 6 + 7;
  a1[25] = (HWAVEOUT)(v29 & 0x7FFFFFF8);
  v30 = 6 * (v29 & 0x7FFFFFF8);
  a1[26] = (HWAVEOUT)v30;
  v31 = (HWAVEOUT)operator new(v30);
  a1[24] = v31;
  if ( v31 == nullptr )
    return 0;
  v32 = 0;
  if ( a1[26] != nullptr )
  {
    do
    {
      v33 = a1[24];
      if ( a1[30] != nullptr )
        *((_BYTE *)v33 + v32) = 0;
      else
        *((_BYTE *)v33 + v32) = 0x80;
      ++v32;
    }
    while ( v32 < (unsigned int)a1[26] );
  }
  InitializeCriticalSection((LPCRITICAL_SECTION)a1 + 2);
  Thread = (HWAVEOUT)CreateThread(nullptr, 0, (LPTHREAD_START_ROUTINE)sub_5ABD8, a1, 4u, (LPDWORD)a1 + 9);
  a1[8] = Thread;
  if ( Thread == nullptr )
    return 0;
  a1[15] = (HWAVEOUT)CreateEventW(nullptr, false, false, nullptr);
  EventW = (HWAVEOUT)CreateEventW(nullptr, false, false, nullptr);
  v36 = a1[18];
  a1[16] = EventW;
  v37 = 0;
  v38 = a1[30];
  *((_BYTE *)a1 + 9) = BYTE1(v36);
  *((_BYTE *)a1 + 10) = BYTE2(v36);
  v39 = a1[29];
  *((_BYTE *)a1 + 11) = HIBYTE(v36);
  *((_BYTE *)a1 + 4) = 1;
  *((_BYTE *)a1 + 5) = 0;
  *((_BYTE *)a1 + 8) = (_BYTE)v36;
  v40 = 1 << (char)v39;
  *((_BYTE *)a1 + 20) = 0;
  *((_BYTE *)a1 + 6) = 1 << (char)v39;
  *((_BYTE *)a1 + 21) = 0;
  *((_BYTE *)a1 + 7) = HIBYTE(v40);
  LOWORD(v38) = 8 << (char)v38;
  *((_WORD *)a1 + 9) = (_WORD)v38;
  *((_WORD *)a1 + 8) = ((unsigned __int16)v38 >> 3) * v40;
  v41 = v40 * (_DWORD)v36;
  *((_WORD *)a1 + 6) = v40 * (_WORD)v36;
  *((_BYTE *)a1 + 14) = BYTE2(v41);
  *((_BYTE *)a1 + 15) = HIBYTE(v41);
  if ( waveOutGetNumDevs() == 0 )
  {
    v42 = v46;
LABEL_50:
    if ( v42 == 0 )
      goto LABEL_53;
    CloseHandle(a1[15]);
    CloseHandle(a1[16]);
    CloseHandle(a1[8]);
    operator delete(a1[20]);
    operator delete(a1[22]);
    operator delete(a1[24]);
    a1[24] = nullptr;
    return 0;
  }
  while ( 1 )
  {
    v42 = waveOutOpen(a1, v37, (LPCWAVEFORMATEX)(a1 + 1), (DWORD)a1[9], (DWORD)a1, 0x20000u);
    if ( v42 == 0 )
      break;
    if ( ++v37 >= waveOutGetNumDevs() )
      goto LABEL_50;
  }
LABEL_53:
  waveOutPause(*a1);
  a1[6] = (HWAVEOUT)operator new(32 * (_DWORD)a1[7]);
  v43 = 0;
  if ( a1[7] != nullptr )
  {
    v44 = 0;
    do
    {
      memset((char *)a1[6] + v44, 0, 0x20u);
      v45 = (char *)a1[6] + v44;
      v45[4] = 8;
      v45[5] = 0;
      v45[6] = 0;
      v45[7] = 0;
      *(_DWORD *)((char *)a1[6] + v44) = (char *)a1[24] + (_DWORD)a1[25] * v43;
      waveOutPrepareHeader(*a1, (LPWAVEHDR)((char *)a1[6] + v44), 0x20u);
      waveOutWrite(*a1, (LPWAVEHDR)((char *)a1[6] + v44), 0x20u);
      ++v43;
      v44 += 32;
    }
    while ( v43 < (unsigned int)a1[7] );
  }
  SetThreadPriority(a1[8], 1);
  Sleep(0x64u);
  ResumeThread(a1[8]);
  waveOutRestart(*a1);
  *((_BYTE *)a1 + 68) = 1;
  return 1;
}

