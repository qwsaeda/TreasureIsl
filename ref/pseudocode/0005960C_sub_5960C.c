/* ea:   0x0005960C */
/* name: sub_5960C */
/* size: 276 */
int __fastcall sub_5960C(int a1)
{
  unsigned int v2; // r5
  int v3; // r4
  void *v4; // r0
  void *v5; // r0
  void *v6; // r0

  if ( *(_BYTE *)(a1 + 68) != 0 )
  {
    ResumeThread(*(HANDLE *)(a1 + 32));
    EventModify(*(HANDLE *)(a1 + 60), 3u);
    WaitForSingleObject(*(HANDLE *)(a1 + 64), 0xFFFFFFFF);
    Sleep(0x64u);
    CloseHandle(*(HANDLE *)(a1 + 60));
    CloseHandle(*(HANDLE *)(a1 + 64));
    CloseHandle(*(HANDLE *)(a1 + 32));
    DeleteCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
    waveOutReset(*(HWAVEOUT *)a1);
    v2 = 0;
    if ( *(_DWORD *)(a1 + 28) != 0 )
    {
      v3 = 0;
      do
      {
        waveOutUnprepareHeader(*(HWAVEOUT *)a1, (LPWAVEHDR)(v3 + *(_DWORD *)(a1 + 24)), 0x20u);
        ++v2;
        v3 += 32;
      }
      while ( v2 < *(_DWORD *)(a1 + 28) );
    }
    waveOutClose(*(HWAVEOUT *)a1);
    if ( *(_DWORD *)(a1 + 24) != 0 )
    {
      operator delete(*(void **)(a1 + 24));
      *(_DWORD *)(a1 + 24) = 0;
    }
    v4 = *(void **)(a1 + 96);
    if ( v4 != nullptr )
    {
      operator delete(v4);
      *(_DWORD *)(a1 + 96) = 0;
    }
    v5 = *(void **)(a1 + 80);
    if ( v5 != nullptr )
    {
      operator delete(v5);
      *(_DWORD *)(a1 + 80) = 0;
    }
    v6 = *(void **)(a1 + 88);
    *(_DWORD *)(a1 + 76) = 0;
    if ( v6 != nullptr )
    {
      operator delete(v6);
      *(_DWORD *)(a1 + 88) = 0;
    }
    *(_DWORD *)(a1 + 84) = 0;
    *(_BYTE *)(a1 + 68) = 0;
  }
  return 1;
}

