/* ea:   0x00062B84 */
/* name: sub_62B84 */
/* size: 160 */
LPARAM __fastcall sub_62B84(_BYTE *a1)
{
  struct tagMSG pMsg; // [sp+4h] [bp-28h] BYREF

  while ( 1 )
  {
    while ( PeekMessageW(&pMsg, nullptr, 0, 0, 1u) )
    {
      if ( pMsg.message == 18 )
        return pMsg.lParam;
LABEL_3:
      DispatchMessageW(&pMsg);
    }
    if ( a1[40] != 0 )
      break;
    (*(void (__fastcall **)(_BYTE *))(*(_DWORD *)a1 + 16))(a1);
  }
  if ( GetMessageW(&pMsg, nullptr, 0, 0) )
    goto LABEL_3;
  return pMsg.lParam;
}

