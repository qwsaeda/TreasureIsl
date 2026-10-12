/* ea:   0x000498A4 */
/* name: sub_498A4 */
/* size: 752 */
int __fastcall sub_498A4(_DWORD *a1)
{
  int result; // r0
  _DWORD *v3; // r4
  __int16 v4; // r7
  int v5; // r3
  _DWORD *i; // r4
  int v7; // r2
  unsigned __int16 v8; // r6
  void *v9; // r0
  int v10; // r0
  int v11; // r5
  int v12; // r3
  unsigned int v13; // r2
  __int16 v14; // r0
  int v15; // r3
  int v16; // r1
  int v17; // r0
  int v18; // [sp+8h] [bp-24h] BYREF
  int v19; // [sp+Ch] [bp-20h] BYREF
  _DWORD v20[7]; // [sp+10h] [bp-1Ch] BYREF

  result = *(unsigned __int16 *)(a1[4] + 16);
  if ( result == 0 )
    return result;
  v3 = (_DWORD *)*sub_30C2C(a1[6] + 496, v20);
  v4 = 0;
  while ( v3 != (_DWORD *)*sub_1DE18((_DWORD *)(a1[6] + 496), v20) )
  {
    if ( (int)v3[4] > 4 )
    {
      v5 = (*(int (__fastcall **)(_DWORD))(*(_DWORD *)v3[5] + 136))(v3[5]);
      result = a1[2];
      if ( v5 == result )
        return result;
    }
    v3 = sub_16A30(v3);
  }
  for ( i = (_DWORD *)*sub_1DDF0(a1[6] + 508, v20); ; i = sub_16A30(i) )
  {
    result = i != (_DWORD *)*sub_1DE08((_DWORD *)(a1[6] + 508), v20);
    if ( (_BYTE)result == 0 )
      return result;
    v7 = i[5];
    if ( *(_BYTE *)(v7 + 50) != 0
      && *(unsigned __int8 *)(v7 + 50) != a1[2]
      && *(unsigned __int8 *)(v7 + 49) < (unsigned int)*(unsigned __int16 *)(a1[4] + 12) )
    {
      break;
    }
  }
  v8 = *(unsigned __int8 *)(i[5] + 49);
  v18 = *(_DWORD *)(a1[6] + 724) + 1;
  v9 = operator new(0xDCu);
  if ( v9 == nullptr )
  {
    v11 = 0;
LABEL_22:
    (*(void (__fastcall **)(int))(*(_DWORD *)v11 + 16))(v11);
    return (**(int (__fastcall ***)(int, int))v11)(v11, 1);
  }
  v10 = sub_55728(v9, 2, *((unsigned __int16 *)i + 8), a1[2], v8, 0);
  v11 = v10;
  if ( v10 == 0
    || (*(unsigned __int8 (__fastcall **)(int, _DWORD, int, int))(*(_DWORD *)v10 + 48))(v10, a1[6], 3, v18) == 0 )
  {
    goto LABEL_22;
  }
  ++*(_DWORD *)(a1[6] + 724);
  v12 = a1[4];
  v13 = *(unsigned __int16 *)(v12 + 12);
  v14 = 0;
  if ( v8 <= v13 )
    v14 = v13 - v8;
  *(_WORD *)(v12 + 12) = v14;
  v15 = a1[4];
  if ( *(_WORD *)(v15 + 16) != 0 )
    v4 = *(_WORD *)(v15 + 16) - 1;
  *(_WORD *)(v15 + 16) = v4;
  *(_DWORD *)sub_1DD10((_DWORD *)(a1[6] + 496), &v18) = v11;
  *(_DWORD *)sub_2FA58((_DWORD *)(a1[6] + 532), &v18) = 56 - *(_DWORD *)(a1[3] + 12);
  *(_DWORD *)(sub_2FA58((_DWORD *)(a1[6] + 532), &v18) + 4) = 94 - *(_DWORD *)(a1[3] + 16);
  v16 = *(_DWORD *)sub_1DD10((_DWORD *)(a1[6] + 496), &v18);
  v17 = a1[6];
  v19 = v16;
  sub_2F770((int *)(v17 + 64), &v19);
  return (int)sub_3958C((_DWORD **)(a1[6] + 64));
}

