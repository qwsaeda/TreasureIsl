/* ea:   0x0003863C */
/* name: sub_3863C */
/* size: 60 */
void __fastcall sub_3863C(void ***a1)
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
  if ( *a1 != nullptr )
    free(*a1);
}

