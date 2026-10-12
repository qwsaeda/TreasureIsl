/* ea:   0x0005987C */
/* name: sub_5987C */
/* size: 64 */
unsigned int __fastcall sub_5987C(int a1)
{
  unsigned int v1; // r5
  unsigned int i; // r3
  unsigned int v3; // r1

  v1 = 0;
  for ( i = 0; i < *(unsigned __int8 *)(a1 + 950); ++i )
  {
    v3 = *(unsigned __int8 *)(i + a1 + 952);
    if ( v1 < v3 )
      v1 = v3;
  }
  return v1 + 1;
}

