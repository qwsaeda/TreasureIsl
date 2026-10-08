/* ea:   0x0003C184 */
/* name: sub_3C184 */
/* size: 324 */
int __fastcall sub_3C184(int a1, int a2, __int16 a3, _DWORD *a4)
{
  _DWORD *i; // r7
  int v8; // r0
  _DWORD *j; // r6
  int v10; // r0
  unsigned int v11; // r0
  char v13; // [sp+0h] [bp-34h] BYREF
  int v14; // [sp+4h] [bp-30h] BYREF
  _BYTE v15[44]; // [sp+8h] [bp-2Ch] BYREF

  sub_3CDA0(v15, &v13);
  for ( i = (_DWORD *)*sub_1DDF0((int)(a4 + 127), &v14); i != (_DWORD *)a4[127]; i = sub_16A30(i) )
  {
    v8 = i[5];
    if ( *(unsigned __int8 *)(v8 + 50) == a3 )
    {
      for ( j = *(_DWORD **)(v8 + 28); j != *(_DWORD **)(v8 + 32); ++j )
      {
        if ( *j != 0 )
        {
          if ( (_DWORD *)(a1 + 36) != dword_81184 )
            sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
          v10 = sub_12F04(aB);
          v11 = sub_3B0C0(dword_81184, (int)aB, 0, v10);
          sub_3AFD8(dword_81184, v11, 2, *(&off_7EDD8 + *j - 1));
          *j = 0;
          sub_34B38(a4, *((unsigned __int16 *)i + 8), 2);
          return sub_3CC14(v15);
        }
      }
    }
  }
  return sub_3CDB4(v15);
}

