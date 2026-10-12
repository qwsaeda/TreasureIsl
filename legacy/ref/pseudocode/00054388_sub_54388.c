/* ea:   0x00054388 */
/* name: sub_54388 */
/* size: 136 */
int __fastcall sub_54388(int a1, int a2, int a3)
{
  int v5; // r6
  int v6; // r5
  int v7; // r4
  int v8; // r5
  int i; // r4

  v5 = a2;
  v6 = (*(_DWORD *)(a1 + 4) - a3) / 36;
  if ( v6 > 0 )
  {
    v7 = a3 - a2;
    do
    {
      sub_49848(v5, (char *)(v7 + v5));
      --v6;
      v5 += 36;
    }
    while ( v6 != 0 );
  }
  v8 = *(_DWORD *)(a1 + 4);
  for ( i = v5; i != v8; i += 36 )
    sub_39E18(i);
  *(_DWORD *)(a1 + 4) = v5;
  return a2;
}

