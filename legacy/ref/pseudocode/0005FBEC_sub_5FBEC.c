/* ea:   0x0005FBEC */
/* name: sub_5FBEC */
/* size: 208 */
int __fastcall sub_5FBEC(int a1, int a2, unsigned int a3)
{
  int v5; // r8
  int v6; // r7
  unsigned int v7; // r6
  char *v8; // r3
  __int16 v9; // t1
  int v11; // [sp+4h] [bp-22Ch] BYREF
  __int16 v12; // [sp+Ah] [bp-226h] BYREF
  _BYTE v13[548]; // [sp+Ch] [bp-224h] BYREF

  if ( a2 == 0 )
    __debugbreak();
  v5 = a3 >> 1;
  v6 = a2;
  v7 = a3 >> 1;
  if ( a3 >> 1 == 0 )
    return 1;
  while ( 1 )
  {
    v11 = 256;
    sub_5FCBC(a1, v6, v7, v13, &v11);
    v8 = (char *)&v12;
    v6 += 2 * v11;
    v7 -= v11;
    do
    {
      v9 = *((_WORD *)v8 + 1);
      v8 += 2;
    }
    while ( v9 != 0 );
    if ( (unsigned int)(v8 - v13) >> 1 != 0 && (unsigned __int8)sub_5FD28(a1, v13, v11) == 0 )
      break;
    if ( v5 <= (v6 - a2) >> 1 )
      return 1;
  }
  sub_196AC((_DWORD *)(a1 + 4));
  return 0;
}

