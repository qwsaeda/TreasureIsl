/* ea:   0x00048AC0 */
/* name: sub_48AC0 */
/* size: 440 */
unsigned int __fastcall sub_48AC0(int a1, int a2, char a3)
{
  int v5; // r5
  _DWORD *v6; // r4
  unsigned int result; // r0
  int v8; // r4
  unsigned int v9; // r2
  int v10; // r1
  unsigned int v11; // r5
  int v12; // r0
  int v13; // r4
  int v14; // [sp+0h] [bp-20h] BYREF
  int v15; // [sp+4h] [bp-1Ch] BYREF

  if ( a2 != 0 && a3 != 0 )
  {
    v5 = 4;
    v6 = (_DWORD *)(a2 + 28);
    while ( 1 )
    {
      if ( v5 >= (unsigned int)((*(_DWORD *)(a2 + 32) - *v6) >> 2) )
        TerminateProcess((HANDLE)0x42, 0);
      if ( *(_DWORD *)(*v6 + 4 * v5) != 0 )
        break;
      if ( --v5 < 0 )
        goto LABEL_11;
    }
    if ( v5 < 5 )
      *(_DWORD *)(*(_DWORD *)(a2 + 28) + 4 * v5) = 0;
LABEL_11:
    result = *(unsigned __int16 *)(a2 + 22);
    if ( *(_WORD *)(a2 + 22) == 0 )
    {
      (*(void (__fastcall **)(_DWORD, _DWORD))(**(_DWORD **)(*(_DWORD *)(a1 + 12) + 40) + 32))(
        *(_DWORD *)(*(_DWORD *)(a1 + 12) + 40),
        *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 8));
      v8 = *(_DWORD *)(a1 + 24) + 144;
      v9 = *(_DWORD *)(*(_DWORD *)(a1 + 24) + 152) - 1;
      *(_DWORD *)(*(_DWORD *)(a1 + 24) + 152) = v9;
      if ( v9 / 7 < v9 / 0xA )
        *(_DWORD *)(v8 + 28) = 0;
      result = *(unsigned __int8 *)(a2 + 48);
      if ( *(_BYTE *)(a2 + 50) == 1 )
      {
        v10 = *(_DWORD *)(a1 + 24);
        if ( result < 0xA )
        {
          --*(_BYTE *)(*(_DWORD *)(v10 + 476) + result);
          result = *(_DWORD *)(v10 + 488);
          *(_DWORD *)(v10 + 488) = result - 1;
          *(_BYTE *)(a2 + 50) = 0;
          return result;
        }
      }
      else
      {
        v11 = *(unsigned __int8 *)(a2 + 48);
        v12 = *(_DWORD *)(a1 + 24);
        v15 = *(unsigned __int8 *)(a2 + 50);
        result = (unsigned int)sub_1EDAC(v12 + 496, &v15);
        v13 = *(_DWORD *)(result + 20);
        if ( v11 < 0xA )
        {
          --*(_BYTE *)(*(_DWORD *)(v13 + 188) + v11);
          result = *(_DWORD *)(v13 + 200);
          *(_DWORD *)(v13 + 200) = result - 1;
        }
      }
      *(_BYTE *)(a2 + 50) = 0;
    }
  }
  else
  {
    v14 = 0;
    sub_5162C(a1, &v14);
    return sub_517EC(a1, 3 * v14, *(unsigned __int8 *)(a2 + 50));
  }
  return result;
}

