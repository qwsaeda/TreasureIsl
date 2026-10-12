/* ea:   0x00030ABC */
/* name: sub_30ABC */
/* size: 20 */
void __fastcall sub_30ABC(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

