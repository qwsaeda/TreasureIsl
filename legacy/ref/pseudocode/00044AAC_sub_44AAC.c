/* ea:   0x00044AAC */
/* name: sub_44AAC */
/* size: 80 */
char **__fastcall sub_44AAC(char ***a1)
{
  char **v2; // r0
  char *v3; // r5
  char *v4; // r4
  char **result; // r0

  v2 = *a1;
  v3 = *v2;
  if ( *v2 != (char *)v2 )
  {
    do
    {
      v4 = v3;
      v3 = *(char **)v3;
      sub_45EFC(v4 + 8);
      free(v4);
    }
    while ( v3 != (char *)*a1 );
  }
  result = *a1;
  **a1 = (char *)*a1;
  (*a1)[1] = (char *)*a1;
  return result;
}

