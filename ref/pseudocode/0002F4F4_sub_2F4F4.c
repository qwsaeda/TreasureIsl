/* ea:   0x0002F4F4 */
/* name: sub_2F4F4 */
/* size: 56 */
int __fastcall sub_2F4F4(void ***a1)
{
  void **v2; // r4
  void **v3; // r5

  v2 = *a1;
  v3 = a1[1];
  if ( *a1 != v3 )
  {
    do
    {
      sub_12F48(v2);
      v2 += 3;
    }
    while ( v2 != v3 );
  }
  return sub_3012C(a1);
}

