/* ea:   0x0005ABD8 */
/* name: sub_5ABD8 */
/* size: 380 */
int __fastcall sub_5ABD8(int a1)
{
  struct wavehdr_tag *lParam; // r5
  DWORD v4; // r3
  CHAR *v5; // r2
  struct tagMSG pMsg; // [sp+4h] [bp-34h] BYREF

  if ( a1 == 0 )
    return -1;
  while ( WaitForSingleObject(*(HANDLE *)(a1 + 60), 5u) != 0 )
  {
    while ( PeekMessageW(&pMsg, nullptr, 0, 0, 1u) )
    {
      if ( pMsg.message == 957 )
      {
        lParam = (struct wavehdr_tag *)pMsg.lParam;
        if ( pMsg.lParam == 0 )
          break;
        *(_DWORD *)(a1 + 112) = *(unsigned __int8 *)pMsg.lParam
                              | (*(unsigned __int8 *)(pMsg.lParam + 1) << 8)
                              | (*(unsigned __int8 *)(pMsg.lParam + 2) << 16)
                              | (*(unsigned __int8 *)(pMsg.lParam + 3) << 24);
        EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
        sub_59C00((unsigned int *)a1);
        LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
        waveOutUnprepareHeader(*(HWAVEOUT *)a1, lParam, 0x20u);
        memset(lParam, 0, sizeof(struct wavehdr_tag));
        v4 = *(_DWORD *)(a1 + 100);
        lParam->dwBufferLength = v4;
        v5 = *(CHAR **)(a1 + 112);
        lParam->lpData = v5;
        waveOutPrepareHeader(*(HWAVEOUT *)a1, lParam, 0x20u);
        waveOutWrite(*(HWAVEOUT *)a1, lParam, 0x20u);
      }
    }
  }
  EventModify(*(HANDLE *)(a1 + 64), 3u);
  return 0;
}

