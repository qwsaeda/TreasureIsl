/* ea:   0x000117D4 */
/* name: sub_117D4 */
/* size: 556 */
int __fastcall sub_117D4(int a1)
{
  int v2; // r0
  int result; // r0
  int v4; // r2
  int v5; // r2

  v2 = *(_DWORD *)(a1 + 1268);
  switch ( v2 )
  {
    case 10:
    case 20:
      return (*(int (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 1272) + 12))(*(_DWORD *)(a1 + 1272));
    case 30:
      (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 1272) + 12))(*(_DWORD *)(a1 + 1272));
      v4 = *(_DWORD *)(a1 + 1272);
      result = *(_DWORD *)(v4 + 1008);
      if ( result != 0 )
      {
        if ( *(_DWORD *)(v4 + 1024) != 0 )
        {
          if ( v4 != 0 )
            (**(void (__fastcall ***)(_DWORD, int))v4)(*(_DWORD *)(a1 + 1272), 1);
          *(_DWORD *)(a1 + 1268) = 0;
          *(_DWORD *)(a1 + 1272) = 0;
          return sub_128D0(a1);
        }
        else
        {
          if ( v4 != 0 )
            (**(void (__fastcall ***)(_DWORD, int))v4)(*(_DWORD *)(a1 + 1272), 1);
          *(_DWORD *)(a1 + 1268) = 0;
          *(_DWORD *)(a1 + 1272) = 0;
          return 0;
        }
      }
      break;
    case 40:
      (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 1272) + 12))(*(_DWORD *)(a1 + 1272));
      v5 = *(_DWORD *)(a1 + 1272);
      result = *(_DWORD *)(v5 + 1008);
      if ( result != 0 )
      {
        if ( *(_DWORD *)(v5 + 1024) != 0 )
        {
          if ( v5 != 0 )
            (**(void (__fastcall ***)(_DWORD, int))v5)(*(_DWORD *)(a1 + 1272), 1);
          *(_DWORD *)(a1 + 1268) = 0;
          *(_DWORD *)(a1 + 1272) = 0;
          return sub_123EC(a1);
        }
        else
        {
          if ( v5 != 0 )
            (**(void (__fastcall ***)(_DWORD, int))v5)(*(_DWORD *)(a1 + 1272), 1);
          *(_DWORD *)(a1 + 1268) = 0;
          *(_DWORD *)(a1 + 1272) = 0;
          return 0;
        }
      }
      break;
    default:
      (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 1020) + 12))(*(_DWORD *)(a1 + 1020));
      sub_11BA0(a1);
      sub_443FC(a1);
      sub_12C50(a1);
      sub_44828(a1, *(unsigned __int8 *)(a1 + 1050));
      sub_44828(a1, *(unsigned __int16 *)(a1 + 1048));
      sub_44828(a1, *(_DWORD *)(a1 + 1024));
      return sub_44828(a1, *(_DWORD *)(a1 + 1028));
  }
  return result;
}

