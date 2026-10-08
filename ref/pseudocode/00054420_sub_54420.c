/* ea:   0x00054420 */
/* name: sub_54420 */
/* size: 60 */
void __fastcall sub_54420(void **a1)
{
  char *v2; // r4
  char *v3; // r5

  v2 = (char *)*a1;
  v3 = (char *)a1[1];
  if ( *a1 != v3 )
  {
    do
    {
      sub_39E18((int)v2);
      v2 += 36;
    }
    while ( v2 != v3 );
  }
  if ( *a1 != nullptr )
    free(*a1);
}

