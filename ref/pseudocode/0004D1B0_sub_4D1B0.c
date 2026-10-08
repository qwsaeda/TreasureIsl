/* ea:   0x0004D1B0 */
/* name: sub_4D1B0 */
/* size: 372 */
int __fastcall sub_4D1B0(int a1, int a2, char a3)
{
  int v5; // r5
  _DWORD *v6; // r4
  int result; // r0
  int v8; // r4
  unsigned int v9; // r2
  unsigned int v10; // r4
  int v11; // r0
  int v12; // r3
  int v13; // [sp+0h] [bp-20h] BYREF
  int v14; // [sp+4h] [bp-1Ch] BYREF

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
      v10 = *(unsigned __int8 *)(a2 + 48);
      v11 = *(_DWORD *)(a1 + 24);
      v14 = *(unsigned __int8 *)(a2 + 50);
      result = (int)sub_1EDAC(v11 + 496, &v14);
      v12 = *(_DWORD *)(result + 20);
      if ( v10 < 0xA )
      {
        --*(_BYTE *)(v10 + *(_DWORD *)(v12 + 188));
        result = *(_DWORD *)(v12 + 200);
        *(_DWORD *)(v12 + 200) = result - 1;
      }
      *(_BYTE *)(a2 + 50) = 0;
    }
  }
  else
  {
    v13 = 0;
    sub_5162C(a1, &v13);
    return sub_517EC(a1, 3 * v13, *(unsigned __int8 *)(a2 + 50));
  }
  return result;
}

