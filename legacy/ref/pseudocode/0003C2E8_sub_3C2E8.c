/* ea:   0x0003C2E8 */
/* name: sub_3C2E8 */
/* size: 224 */
int __fastcall sub_3C2E8(int a1, int a2, __int16 a3, _DWORD *a4)
{
  _DWORD *i; // r3
  unsigned __int16 *v8; // r5
  int v9; // r0
  _DWORD *v10; // r2
  _DWORD *j; // r1
  char v13; // [sp+0h] [bp-2Ch] BYREF
  int v14; // [sp+4h] [bp-28h] BYREF
  _BYTE v15[36]; // [sp+8h] [bp-24h] BYREF

  sub_3CDA0(v15, &v13);
  for ( i = (_DWORD *)*sub_1DDF0((int)(a4 + 127), &v14); i != (_DWORD *)a4[127]; i = sub_16A30(i) )
  {
    if ( *(unsigned __int8 *)(i[5] + 50) == a3 )
    {
      v8 = (unsigned __int16 *)(i + 4);
      v9 = i[5];
      v10 = *(_DWORD **)(v9 + 32);
      for ( j = *(_DWORD **)(v9 + 28); j != v10; ++j )
        *j = 0;
      if ( (_DWORD *)(a1 + 36) != dword_81184 )
        sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
      sub_34B38(a4, *v8, 2);
      return sub_3CDB4(v15);
    }
  }
  return sub_3CDB4(v15);
}

