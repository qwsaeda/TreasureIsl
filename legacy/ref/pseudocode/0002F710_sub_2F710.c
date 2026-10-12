/* ea:   0x0002F710 */
/* name: sub_2F710 */
/* size: 56 */
int __fastcall sub_2F710(int *a1)
{
  int v2; // r4
  int v3; // r5

  v2 = *a1;
  v3 = a1[1];
  if ( *a1 != v3 )
  {
    do
    {
      sub_39E18(v2);
      v2 += 36;
    }
    while ( v2 != v3 );
  }
  return sub_30A68(a1);
}

