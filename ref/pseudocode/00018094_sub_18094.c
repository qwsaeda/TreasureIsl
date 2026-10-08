/* ea:   0x00018094 */
/* name: sub_18094 */
/* size: 96 */
int __fastcall sub_18094(unsigned __int16 *a1)
{
  unsigned __int16 *v1; // r4
  int v2; // r5
  char *v3; // r3
  __int16 v4; // t1
  unsigned int i; // r3
  unsigned __int16 v6; // t1

  v1 = a1;
  if ( a1 == nullptr )
    __debugbreak();
  v2 = 0;
  v3 = (char *)(a1 - 1);
  do
  {
    v4 = *((_WORD *)v3 + 1);
    v3 += 2;
  }
  while ( v4 != 0 );
  for ( i = (unsigned int)(v3 - (char *)a1) >> 1; i != 0; v2 = 33 * v2 + v6 )
  {
    v6 = *v1++;
    --i;
  }
  return v2;
}

